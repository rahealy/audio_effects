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

use shared::info::About;
use shared::processor::{Processor, Info, Blocks, Process, SampleType};
use shared::block::{Input, Output, Buffers};
use shared::buffer::BUFFER_LEN;

static SINE_TAU: SampleType = (2.0 * 3.14159265358979);

#[derive(Default)]
pub struct Sine {
    cnt:        SampleType,
    pub freq:   Input,
    pub smplrt: Input,
    pub scale:  Input,
    pub offset: Input,
    output:     Output
}

impl Processor for Sine {}

impl Process for Sine {
    fn process(& mut self) -> &mut dyn Processor
    {
        for _i in 0..BUFFER_LEN {
            let freq   = self.freq.sum_next();
            let smplrt = self.smplrt.sum_next();
            let scale  = self.scale.sum_next();
            let offset = self.offset.sum_next(); 

            self.cnt += 1.0;
            if self.cnt > smplrt {
                self.cnt = 1.0;
            }

            self.output.put (
                (SampleType::sin(SINE_TAU * freq * self.cnt / smplrt) * scale) + offset
            );
        }
        self
    }

///
///Default values are 440 Hz (A4), 44100kHz (CD Quality) sample rate
///scale by 1.0 (no scaling) and add an offset of 0.0 (no offset).
///
    fn reset(& mut self) -> &mut dyn Processor {
        self.cnt = 0.0;
        self.freq.fill_split(1, 440.0, 0.0);
        self.smplrt.fill_split(1, 44100.0, 0.0);
        self.scale.fill_split(1, 1.0, 0.0);
        self.offset.fill(0.0);
        return self;
    }
}

impl Blocks for Sine {
    fn input(&mut self, idx: usize) -> &mut Input {
        match idx {
            0 => &mut self.freq,
            1 => &mut self.smplrt,
            2 => &mut self.scale,
            3 => &mut self.offset,
            _ => panic!("Index out of bounds.")
        }
    }

    fn output(&mut self, idx: usize) -> &mut Output {
        match idx {
            0 => &mut self.output,
            _ => panic!("Index out of bounds.")
        }
    }

    fn map_inputs(& mut self, f: & mut dyn FnMut(&mut Input) -> bool) -> bool {
        if f(&mut self.freq) {
            if f(&mut self.smplrt) {
                if f(&mut self.scale) {
                    return f(&mut self.offset);
                }
            }
        }
        return false;
    }

    fn map_outputs(& mut self, f: & mut dyn FnMut(&mut Output) -> bool) -> bool {
        return f(&mut self.output);
    }
}

impl Info for Sine {
    fn info(&self) -> &'static About {
        return &About {
            name: "Sine Wave Generator",
            desc: "Generates sine waves."
        }
    }

    fn num_inputs(&self) -> usize { 4 }

    fn num_outputs(&self) -> usize { 1 }

    fn input_info(&self, idx:usize) -> &'static About {
        match idx {
            0 => & About {
                name: "Frequency",
                desc: "Frequency of the sine wave in Hz"
            },
            
            1 => & About {
                name: "Sample Rate",
                desc: "Sample rate in samples per second"
            },

            2 => & About {
                name: "Scale",
                desc: "Scale output"
            },

            3 => & About {
                name: "Offset",
                desc: "Add offset after output has been scaled"
            },

            _ => panic!("Index out of bounds.")
        }
    }

    fn output_info(&self, idx: usize) -> &'static About {
        match idx {
            0 => & About {
                name: "Output",
                desc: "Sine wave output."
            },

            _ => panic!("Index out of bounds.")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sine::{Sine};
    use shared::processor::{Processor, Process, Blocks};

    #[test]
    fn sine() {
        let mut s = Sine::default();
        for _i in 0..2 {
            s.reset()
             .process();
        }
    }
}
