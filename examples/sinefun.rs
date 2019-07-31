/*
MIT License

Copyright (c) 2019 Richard A. Healy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use rack::unit::{Unit};
use effects::sine;
use effects::fout;
use shared::processor::{Process, Blocks, Processor};
use shared::connector::{Connector, Connection, EndPoint};
use shared::block::{Buffer, Buffers, Connectors};
use shared::buffer::{BUFFER_LEN};
use std::fs::File;

static fname_fout0: &'static str = "sinefun.raw";

fn main() {
    let mut rackunit = Unit::default();
    let mut sine0 = sine::Sine::default();
    let mut sine1 = sine::Sine::default();
    let mut sine2 = sine::Sine::default();
    let mut sine3 = sine::Sine::default();
    let mut fout0 = fout::FOut::default();

    sine0.reset();
    sine1.reset();
    sine2.reset();
    sine3.reset();
    fout0.reset();

//Sine0 is going to modulate the amplitude of sine3 at a frequency of 4Hz. 
    sine0.freq.fill_split  (1, 4.0,  0.0); //Frequency
    sine0.scale.fill_split (1, 0.10, 0.0); //Scale

//Sine1 is going to modulate the amplitude of sine3 at a frequency of 8Hz.
    sine1.freq.fill_split  (1, 8.0,  0.0); //Frequency
    sine1.scale.fill_split (1, 0.10, 0.0); //Scale

//Sine2 is going to modulate the pitch of sine3 at a frequency of 3Hz centered at 440Hz.
    sine2.freq.fill_split  (1, 3.0,   0.0); //Frequency
    sine2.scale.fill_split (1, 0.75,  0.0); //Scale
    sine2.offset.fill_split(1, 440.0, 0.0); //Offset

//Open file for fout0.
    if let Ok(f) = File::create(fname_fout0) {
        println!("Successfully opened: {}", fname_fout0);
        fout0.file(f);
    } else {
        panic!("Couldn't open file: {}", fname_fout0);
    }

//Rack em' up.
    rackunit.add(&mut sine0);
    rackunit.add(&mut sine1);
    rackunit.add(&mut sine2);
    rackunit.add(&mut sine3);
    rackunit.add(&mut fout0);

//Connect output of sine0 to scale of sine3, connector 0.
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 0, block: 0, conn: 0},
            to:   EndPoint {proc: 3, block: 2, conn: 0}
        }
    ) { panic!(e); }

//Connect output of sine1 to scale of sine3, connector 1.
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 1, block: 0, conn: 1},
            to:   EndPoint {proc: 3, block: 2, conn: 1}
        }
    ) { panic!(e); }

//Connect output of sine2 to frequency of sine3, connector 0.
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 2, block: 0, conn: 0},
            to:   EndPoint {proc: 3, block: 0, conn: 0}
        }
    ) { panic!(e); }

//Connect output of sine3 to input of fout0, connector 0.
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 3, block: 0, conn: 0},
            to:   EndPoint {proc: 4, block: 0, conn: 0}
        }
    ) { panic!(e); }

//Start processing.
    rackunit.start();

//Default sample rate for sine is 44100kHz. Process enough times to 
//roughly generate 1 second's worth of samples.
    for _i in 0..(44100 / BUFFER_LEN + 1) * 5 {
        rackunit.process_next();
        rackunit.dispatch_next_forward();
        rackunit.dispatch_backward();
    }

    rackunit.drain_and_stop();
}
