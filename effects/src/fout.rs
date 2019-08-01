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
use shared::processor::{Processor, Info, Blocks, Process};
use shared::block::{Input, Output, Buffers};
use shared::buffer::BUFFER_LEN;
use std::fs::File;
use std::io::Write;

pub enum FileHandle {
    IsOpen(File),
    Closed
}

impl Default for FileHandle {
    fn default() -> FileHandle {
        FileHandle::Closed
    }
}

#[derive(Default)]
pub struct FOut {
    file: FileHandle,
    input: Input
}

impl FOut {
    pub fn file(&mut self, f: File) {
        self.file = FileHandle::IsOpen(f);
    }
}

impl Processor for FOut {}

impl Process for FOut {
    fn process(& mut self) -> &mut dyn Processor
    {
//        println!("fout::process(): HERE!");
        if let FileHandle::IsOpen(f) = &mut self.file {
            for _ in 0..BUFFER_LEN {
                let bytes = self.input
                                .sum_next()
                                .to_bits()
                                .to_ne_bytes();

                if let Err(err) = f.write_all(&bytes) {
                    panic!("fout.process(): {}", err);
                }
            }
        }
        self
    }

    fn reset(& mut self) -> &mut dyn Processor { 
        if let FileHandle::IsOpen(f) = &self.file {
            drop(f);
            self.file = FileHandle::Closed;
        }
        self
    }
}

impl Blocks for FOut {
    fn input(&mut self, idx: usize) -> &mut Input {
        match idx {
            0 => &mut self.input,
            _ => panic!("Index out of bounds.")
        }
    }

    fn output(&mut self, _idx: usize) -> &mut Output {
        panic!("FOut doesn't have any outputs.")
    }

    fn map_inputs(& mut self, f: & mut dyn FnMut(&mut Input) -> bool) -> bool {
        return f(&mut self.input);
    }
}

impl  Info for FOut {
    fn info(&self) -> &'static About {
        return &About {
            name: "File Output",
            desc: "Writes input to a raw file."
        }
    }

    fn num_inputs(&self) -> usize { 1 }

    fn num_outputs(&self) -> usize { 0 }

    fn input_info(&self, idx:usize) -> &'static About {
        match idx {
            0 => & About {
                name: "Input",
                desc: "Input data is summed and written to file."
            },
            _ => panic!("Index out of bounds.")
        }
    }

    fn output_info(&self, idx: usize) -> &'static About {
        match idx {
            _ => panic!("Index out of bounds.")
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fout() {
    }
}
 
