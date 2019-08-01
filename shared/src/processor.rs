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


/**********************************************************************
 * SampleType
 *********************************************************************/

use crate::info::About;
use crate::block::{Input, Output};

///
/// Process sample type.
///
pub type SampleType = f32;

/**********************************************************************
 * Processor
 *********************************************************************/

///
///Processor traits are broken into the following primary groups:
/// Process - Everything to do with processor state.
/// Blocks - Provides access to the processor's I/O blocks.
/// Info - Provides information about the processor.
/// 
pub trait Processor: Info + Blocks + Process {}

pub trait Process: Info + Blocks {
    fn process(& mut self) -> &mut dyn Processor;  //Process the data.
    fn reset(& mut self) -> &mut dyn Processor; //Reset the processor to defaults.
}

pub trait Blocks {
    fn output(&mut self, idx: usize) -> &mut Output;
    fn input(&mut self, idx: usize) -> &mut Input;
    fn map_inputs(&mut self, _f: &mut dyn FnMut(&mut Input) -> bool) -> bool { false }
    fn map_outputs(&mut self, _f: &mut dyn FnMut(&mut Output) -> bool) -> bool { false }
}

pub trait Info {
    fn info(&self) -> &'static About;
    fn input_info(&self, idx: usize) -> &'static About;
    fn output_info(&self, idx: usize) -> &'static About;
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    
    fn map_input_info(&self, f: &mut dyn FnMut(&'static About) -> bool) -> bool { 
        for i in 0..self.num_inputs() {
            if !f(self.input_info(i)) {
                return false;
            }
        }
        return true;
    }

    fn map_output_info(&self, f: &mut dyn FnMut(&'static About) -> bool) -> bool {
        for i in 0..self.num_outputs() {
            if !f(self.output_info(i)) {
                return false;
            }
        }
        return true;
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn processor() {
    }
}
