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

#[derive(Default)]
pub struct Pwm {
    cnt:        SampleType,
    pub freq:   Input,
    pub smplrt: Input,
    pub scale:  Input,
    pub offset: Input,
    pub duty:   Input,
    output:     Output
}

impl Processor for Pwm {}

impl Process for Pwm {
    fn process(& mut self) -> &mut dyn Processor
    {
        for _i in 0..BUFFER_LEN {
            let freq   = self.freq.sum_next();
            let smplrt = self.smplrt.sum_next();
            let scale  = self.scale.sum_next();
            let offset = self.offset.sum_next();
            let duty   = self.duty.sum_next();

            self.cnt += 1.0;
            if self.cnt > smplrt {
                self.cnt = 1.0;
            }

            let spc: SampleType = smplrt / freq;            //Samples per cycle
            let phase: SampleType = (self.cnt % spc) / spc; //Phase in percentage - 0..1
            let smpl_out: SampleType = if phase > duty { -1.0 } else { 1.0 };

            self.output.put(smpl_out * scale + offset);
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
        self.duty.fill_split(1, 0.5, 0.0);
        return self;
    }
}

impl Blocks for Pwm {
    fn input(&mut self, idx: usize) -> &mut Input {
        match idx {
            0 => &mut self.freq,
            1 => &mut self.smplrt,
            2 => &mut self.scale,
            3 => &mut self.offset,
            4 => &mut self.duty,
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
                    if f(&mut self.duty) {
                        return f(&mut self.offset);
                    }
                }
            }
        }
        return false;
    }

    fn map_outputs(& mut self, f: & mut dyn FnMut(&mut Output) -> bool) -> bool {
        return f(&mut self.output);
    }
}

impl Info for Pwm {
    fn info(&self) -> &'static About {
        return &About {
            name: "Pulse Width Modulation",
            desc: "Generates pulse width modulated square waves."
        }
    }

    fn num_inputs(&self) -> usize { 5 }

    fn num_outputs(&self) -> usize { 1 }

    fn input_info(&self, idx:usize) -> &'static About {
        match idx {
            0 => & About {
                name: "Frequency",
                desc: "Frequency of one pwm cycle in Hz"
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

            4 => & About {
                name: "Duty",
                desc: "Percentage of time-on"
            },

            _ => panic!("Index out of bounds.")
        }
    }

    fn output_info(&self, idx: usize) -> &'static About {
        match idx {
            0 => & About {
                name: "Output",
                desc: "Pulse width modulation output."
            },
            _ => panic!("Index out of bounds.")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pwm::{Pwm};
    use shared::processor::{Processor, Process, Blocks};

    #[test]
    fn pwm() {
        let mut s = Pwm::default();
        for _i in 0..2 {
            s.reset()
             .process();
        }
    }
}
 
