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

use audio_effects::prelude::*;
use std::fs::File;

static fname_fout0: &'static str = "sinefun.raw";

fn print_processor_info(proc: &mut dyn Processor) {
    let mut print_about = |about: &About| -> bool {
        print!("  ");
        println!("{} - {}", about.name, about.desc);
        true
    };

    println!("{}", proc.info().name);
    println!(" {}", proc.info().desc);
    println!("");

    println!(" Inputs:");
    proc.map_input_info(&mut print_about);
    if proc.num_inputs() == 0 {
        println!("  None.");
    }

    println!("");

    println!(" Outputs:");
    proc.map_output_info(&mut print_about);
    if proc.num_outputs() == 0 {
        println!("  None.");
    }

    println!("");
}


fn main() {
    let mut rackunit = Unit::default();
    let mut sine0 = sine::Sine::default();
    let mut sine1 = sine::Sine::default();
    let mut sine2 = sine::Sine::default();
    let mut sine3 = sine::Sine::default();
    let mut fout0 = fout::FOut::default();

    println!("");
    println!("sinefun");
    println!(" Copyright (C) 2019 Richard A. Healy");
    println!(" An example demonstrating the use of the audio_effects library.");
    println!("");

    println!("***Initialization***");
    sine0.reset();
    sine1.reset();
    sine2.reset();
    sine3.reset();
    fout0.reset();

//Open file for fout0.
    if let Ok(f) = File::create(fname_fout0) {
        println!("Successfully opened: {}", fname_fout0);
        println!("");
        fout0.file(f);
    } else {
        panic!("fout0: Couldn't open file: {}", fname_fout0);
    }

//Rack em' up.
    rackunit.add(&mut sine0); //0
    rackunit.add(&mut sine1); //1
    rackunit.add(&mut sine2); //2
    rackunit.add(&mut sine3); //3
    rackunit.add(&mut fout0); //4

//Print information about the processors.
    println!("***Meet The Processors***");
    print_processor_info(rackunit.processor(0)); //sine0
    print_processor_info(rackunit.processor(4)); //fout0
 
    println!("***Configure The Processors***");

    println!("sine0: Modulates the amplitude of sine3 at a frequency of 4Hz."); 
    rackunit.processor(0).input(0).fill_split (1, 4.0,  0.0); //Frequency
    rackunit.processor(0).input(2).fill_split (1, 0.10, 0.0); //Scale

    println!("sine1: Modulates the amplitude of sine3 at a frequency of 8Hz.");
    rackunit.processor(1).input(0).fill_split (1, 8.0,  0.0); //Frequency
    rackunit.processor(1).input(2).fill_split (1, 0.10, 0.0); //Scale

    println!("sine2: Modulates the pitch of sine3 at a frequency of 3Hz centered at 440Hz.");
    rackunit.processor(2).input(0).fill_split (1, 3.0,   0.0); //Frequency
    rackunit.processor(2).input(2).fill_split (1, 0.75,  0.0); //Scale
    rackunit.processor(2).input(3).fill_split (1, 440.0, 0.0); //Offset

    println!("");
    println!("***Connect The Processors***");

    println!("Connect output of sine0 to scale of sine3, connector 0.");
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 0, block: 0, conn: 0},
            to:   EndPoint {proc: 3, block: 2, conn: 0}
        }
    ) { panic!(e); }

    println!("Connect output of sine1 to scale of sine3, connector 1.");
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 1, block: 0, conn: 1},
            to:   EndPoint {proc: 3, block: 2, conn: 1}
        }
    ) { panic!(e); }

    println!("Connect output of sine2 to frequency of sine3, connector 0.");
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 2, block: 0, conn: 0},
            to:   EndPoint {proc: 3, block: 0, conn: 0}
        }
    ) { panic!(e); }

    println!("Connect output of sine3 to input of fout0, connector 0.");
    if let Err(e) = rackunit.connect (
        Connection {
            from: EndPoint {proc: 3, block: 0, conn: 0},
            to:   EndPoint {proc: 4, block: 0, conn: 0}
        }
    ) { panic!(e); }

    println!("");
    println!("***Start Processing***");
    rackunit.start();

//Default sample rate for sine is 44100kHz. Process enough times to 
//generate roughly 1 second's worth of samples.
    for _i in 0..(44100 / BUFFER_LEN + 1) * 5 { //There are 5 processors in graph.
        rackunit.process_next();
        rackunit.dispatch_next_forward();
        rackunit.dispatch_backward();
    }

    println!("***Stop Processing***");
    println!("");

    rackunit.drain_and_stop();
}
