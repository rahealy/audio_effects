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


use crate::processor::SampleType;
use crate::buffer;
use crate::buffer::{Read,Write};
use crate::connector::{Connection, Connector};

pub const BLOCK_LEN: usize = 8;
pub type Buffer = buffer::Buffer<SampleType>;


/**********************************************************************
 * Buffers
 *********************************************************************/

///
///Various things that are handy to do with a fixed length block of 
///buffers.
///
pub trait Buffers {

///
/// For each buffer in the block get the next sample and add it to the
/// rest.
///
    fn sum_next(&mut self) -> SampleType {
        let mut s = SampleType::default();
        for x in self.buffers().iter_mut() { s += x.next() };
        return s;
    }

///
/// Write a single sample to all buffers in the block.
///
    fn put(&mut self, val: SampleType) -> () {
        for x in self.buffers().iter_mut() { x.put(val) };
    }

///
/// Fill all buffers in the block with a single sample value.
///
    fn fill(&mut self, val: SampleType) -> () {
        for x in self.buffers().iter_mut() { x.fill(val) };
    }

///
/// Fill a slice of buffers in the block with a single sample value.
///
    fn fill_slice(&mut self, 
                  beg:usize, 
                  len:usize, 
                  val: SampleType) -> () 
    {
        for x in self.buffers()[beg..beg+len].iter_mut() { 
            x.fill(val);
        }
    }

///
/// Given two values fill the buffers up to but not including idx with
/// the first value and then from idx to the end of the list with the
/// second value.
/// 
    fn fill_split(&mut self, 
                  idx:usize, 
                  l_val: SampleType, 
                  r_val: SampleType) -> () 
    {
        for buf in self.buffers()[..idx].iter_mut() {
            buf.fill(l_val);
        }

        for buf in self.buffers()[idx..].iter_mut() {
            buf.fill(r_val);
        };
    }

///
/// Accessor.
///
    fn buffers(&mut self) -> &mut [Buffer; BLOCK_LEN];

    fn buffer(&mut self, idx: usize) -> &mut Buffer {
        &mut self.buffers()[idx]
    }
}


/**********************************************************************
 * Connectors
 *********************************************************************/

///
///Various things that are handy to do with the connectors in a fixed
///length block of buffers.
///
pub trait Connectors {
///
///Make connection from self to specified buffer.
///
    fn connect(&mut self, con: Connection) -> Result<(),&'static str> {
        if let Connector::Unconnected = self.connectors()[con.from.conn] {
            self.connectors()[con.from.conn] = Connector::ConnectedUsing(con);
            self.inc_num_cons();
            Ok(())
        } else {
            Err("block.connect(): End point is already connected.")
        }
    }

///
///Break connection from self to specified buffer.
///
    fn disconnect(&mut self, idx: usize) -> Result<(), &'static str> {
        if let Connector::ConnectedUsing(_) = self.connectors()[idx] {
            self.connectors()[idx] = Connector::Unconnected;
            self.dec_num_cons();
            Ok(())
        } else {
            Err("block.connect(): End point is not connected.")
        }
    }

///
/// Accessor for connector list.
///
    fn connectors(&mut self) -> &mut [Connector; BLOCK_LEN];

///
/// Accessor for a single connector.
///
    fn connector(&mut self, idx: usize) -> &mut Connector {
        &mut self.connectors()[idx]
    }

///
///Keep track of the number of connections.
///
    fn num_cons(&self) -> usize;
    fn inc_num_cons(&mut self);
    fn dec_num_cons(&mut self);
}


/**********************************************************************
 * Block
 *********************************************************************/

///
///A fixed length block of buffers with corresponding connectors to 
///other block buffers.
///
#[derive(Default)]
pub struct Block {
    pub bufs:  [Buffer; BLOCK_LEN],
    pub conns: [Connector; BLOCK_LEN],
    pub num_cons: usize
}


/**********************************************************************
 * Input Block
 *********************************************************************/

#[derive(Default)]
pub struct Input {
    pub b: Block,
    pub full_cnt: usize
}

impl Buffers for Input {
    fn buffers(&mut self) -> &mut [Buffer; BLOCK_LEN] {
        &mut self.b.bufs
    }
}

impl Connectors for Input {
    fn connectors(&mut self) -> &mut  [Connector; BLOCK_LEN] {
        &mut self.b.conns
    }

    fn num_cons(&self) -> usize { self.b.num_cons }
    fn inc_num_cons(&mut self) { self.b.num_cons += 1; }
    fn dec_num_cons(&mut self) { self.b.num_cons -= 1; }
}

impl Input {
    pub fn inc_full_cnt(&mut self) -> () {
        self.full_cnt += 1;
    }

    pub fn full_cnt(&self) -> usize {
        self.full_cnt
    }
    
    pub fn rst_full_cnt(&mut self) -> () {
        self.full_cnt = 0;
    }
}

/**********************************************************************
 * Output Block
 *********************************************************************/

#[derive(Default)]
pub struct Output {
    pub b: Block,
    pub empty_cnt: usize
}

impl Buffers for Output {
    fn buffers(&mut self) -> &mut [Buffer; BLOCK_LEN] {
        &mut self.b.bufs
    }
}

impl Connectors for Output {
    fn connectors(&mut self) -> &mut  [Connector; BLOCK_LEN] {
        &mut self.b.conns
    }

    fn num_cons(&self) -> usize { self.b.num_cons }
    fn inc_num_cons(&mut self) { self.b.num_cons += 1; }
    fn dec_num_cons(&mut self) { self.b.num_cons -= 1; }
}


impl Output {
    pub fn inc_empty_cnt(&mut self) -> () {
        self.empty_cnt += 1;
    }

    pub fn empty_cnt(&self) -> usize {
        self.empty_cnt
    }
    
    pub fn rst_empty_cnt(&mut self) -> () {
        self.empty_cnt = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{Block};

    #[test]
    fn block() {
        let blk = Block::default();
    }
}
