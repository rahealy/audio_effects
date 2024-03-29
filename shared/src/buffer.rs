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


///
/// Fixed length Buffer.
///

/**********************************************************************
 * Buffer
 *********************************************************************/

pub const BUFFER_LEN: usize = 256;

pub trait Size {
   fn size(&self) -> usize { BUFFER_LEN }
}

pub trait Read<T> {
    fn next(& mut self) -> T;
    fn rewind(& mut self) -> ();
    fn empty(& mut self) -> bool;
    fn rdpos(&self) -> usize;
}

pub trait Write<T> {
    fn put(&mut self, val:T) -> ();
    fn fill(&mut self, val:T) -> ();
    fn reset(& mut self) -> ();
    fn full(& mut self) -> bool;
    fn wrpos(&self) -> usize;
    fn copy_from(&mut self, from: &Buffer<T>) -> ();
}

pub trait BufferTrait<I>: Read<I> + Write<I> where
    I: Copy + Clone + Default
{}

#[derive(Clone, Copy)]
pub struct Buffer<S> {
    rdpos: usize,
    wrpos: usize,
    buf: [S; BUFFER_LEN]
}

impl <I> Default for Buffer<I> where
    I: Copy + Clone + Default
{
    fn default() -> Buffer<I> {
        Buffer::<I> {
            rdpos: 0,
            wrpos: 0,
            buf: [I::default(); BUFFER_LEN]
        }
    }
}

impl <I> Size for Buffer<I> where 
    I: Copy + Clone + Default 
{}

impl <I> Read<I> for Buffer<I> where
    I: Copy + Clone + Default
{
    fn next(& mut self) -> I {
        let idx = self.rdpos;

        if idx == self.wrpos {
            if idx == 0 {
                I::default()
            } else {
                self.buf[idx - 1]
            }
        } else {
            self.rdpos += 1;
            self.buf[idx]
        }
    }

    fn rewind(& mut self) -> () {
        self.rdpos = 0;
    }
    
    fn empty(& mut self) -> bool {
        (self.rdpos == self.wrpos)
    }
    
    fn rdpos(&self) -> usize { 
        self.rdpos
    }
}


impl <I> Write<I> for Buffer<I> where
    I: Copy + Clone + Default
{
    fn put(&mut self, val:I) -> () {
        if self.full() {
            self.buf[BUFFER_LEN - 1] = val;
        } else {
            self.buf[self.wrpos] = val;
            self.wrpos += 1;
        }
    }

    fn fill(&mut self, val:I) -> () {
        for i in 0..BUFFER_LEN {
            self.buf[i] = val;
        }
        self.rdpos = 0;
        self.wrpos = BUFFER_LEN;
    }

    fn reset(& mut self) -> () {
        self.rdpos = 0;
        self.wrpos = 0;
    }

    fn full(& mut self) -> bool {
        (self.wrpos == BUFFER_LEN)
    }

    fn copy_from(&mut self, from: &Buffer<I>) -> () {
        self.buf = from.buf;
        self.rdpos = 0;
        self.wrpos = BUFFER_LEN;
    }

    fn wrpos(&self) -> usize { 
        self.wrpos
    }
}

impl <I> BufferTrait<I> for Buffer<I> where
    I: Copy + Clone + Default
{}

///
/// Maps a function to two buffers contained in an array slice where 
///
/// bufs - array slice of buffers that f will be applied to.
/// left - Non-zero length array slice of indexes of left buffers.
/// right - Non-zero length array slice of indexes of right buffers.
/// dest - Non-zero length array slice of indexes of destination buffers.
//
/// f - fn(left,right) -> dest
///  left - value of left buffer at current index.
///  right - value of right buffer at current index.
///
///  Returns value put into destination buffer at currenzt index.
///
/// examples:
///  Given four buffers sum first with third, second with forth and put
///  result in first and second.
///
///  distribute(bufs, [0,1], [2,3], |d,s| d + s);
///
pub fn apply<F: Copy> (bufs:  &mut[Buffer<F>], 
                       left:  &[usize],
                       right: &[usize],
                       dest:  &[usize],
                       f:     fn(F,F) -> F) -> ()
{
    let mut maxlen:usize = 0;

    if maxlen < left.len()  { maxlen = left.len();  }
    if maxlen < right.len() { maxlen = right.len(); }
    if maxlen < dest.len()  { maxlen = dest.len();  }
    
    for i in 0..maxlen {
        let l_idx = left[i % left.len()] % bufs.len();
        let r_idx = right[i % right.len()] % bufs.len();
        let d_idx = dest[i % dest.len()] % bufs.len();

        for j in 0..BUFFER_LEN {
            bufs[d_idx].buf[j] = f(bufs[l_idx].buf[j], bufs[r_idx].buf[j]);
        }
        bufs[d_idx].rdpos = 0;
        bufs[d_idx].wrpos = BUFFER_LEN;
    }
}

///
/// Iterates across an array slice of buffers and applies a function 
/// f(dstval,srcval) -> dst to the values in the current buffer. The 
/// source buffer in the buffer slice is determined by the 
/// corresponding value in map. 
///
/// bufs - array slice of buffers that f will be applied to.
/// map - Non-zero length array slice of indexes of source buffers.
//
/// f - fn(dst,src) -> val
///  dst - value of destination buffer at current index.
///  src - value of source buffer at current index.
///
/// Returns value put into destination buffer at current index.
///
///
/// examples:
///  Given four buffers sum first with third, second with forth and put
///  result in first and second.
///
///  distribute(bufs,[2,3], |d,s| d + s);
///
pub fn distribute<F: Copy> (bufs: &mut[Buffer<F>], 
                            map:  &[usize],
                            f:    fn(F,F) -> F) -> ()
{
    for dstidx in 0..bufs.len() {
        let srcidx = map[dstidx % map.len()];
        if dstidx != srcidx {
            for i in 0..BUFFER_LEN {
                bufs[dstidx].buf[i] = f(bufs[dstidx].buf[i], bufs[srcidx].buf[i]);
            }
            bufs[dstidx].rdpos = 0;
            bufs[dstidx].wrpos = BUFFER_LEN;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::{Buffer, Read, Write, Size};
    use crate::buffer::{distribute, apply};
    use crate::buffer::BUFFER_LEN;

    #[test]
    fn buffer() {
        let mut buf = Buffer::<f32>::default();
//test put(), next()
        for k in 0..buf.size() {
            buf.put(k as f32);
            assert!(buf.wrpos() == k + 1);
        }
        for k in 0..buf.size() {
            assert!(buf.next() == k as f32);
            assert!(buf.rdpos() == k + 1);
        }
//test rewind
        buf.rewind();
        assert!(buf.rdpos() == 0);
        assert!(buf.wrpos() == buf.size());

        for k in 0..buf.size() {
            assert!(buf.next() == k as f32);
            assert!(buf.rdpos() == k + 1);
        }
//test reset
        buf.reset();
        assert!(buf.rdpos() == 0);
        assert!(buf.wrpos() == 0);
    }

    #[test]
    fn buffers() {
        let fill = [98.6, 96.8, 89.6];
        let mut bufs = [Buffer::<f32>::default(), 
                        Buffer::<f32>::default(),
                        Buffer::<f32>::default()];

//fill() and distribute()
        for i in 0..bufs.len() {
            bufs[i].fill(fill[i]);
            assert!(bufs[i].rdpos == 0);
            assert!(bufs[i].wrpos == BUFFER_LEN);

            for j in 0..BUFFER_LEN {
                assert!(bufs[i].buf[j] == fill[i]);
            }
        }

        distribute(& mut bufs, &[1], |_,b| b);
        for i in 0..bufs.len() {
            for j in 0..BUFFER_LEN {
                assert!(bufs[i].buf[j] == fill[1]);
            }
        }

//fill() and distribute()
        for i in 0..bufs.len() {
            bufs[i].fill(fill[i]);
        }

        distribute(& mut bufs, &[1,2], |_,b| b);
        for i in 0..BUFFER_LEN {
            assert!(bufs[0].buf[i] == fill[1]);
            assert!(bufs[1].buf[i] == fill[2]);
            assert!(bufs[2].buf[i] == fill[2]);
        }

//fill() and distribute()
        for i in 0..bufs.len() {
            bufs[i].fill(fill[i]);
        }

        distribute(& mut bufs, &[1,2,2], |_,b| b);
        for i in 0..BUFFER_LEN {
            assert!(bufs[0].buf[i] == fill[1]);
            assert!(bufs[1].buf[i] == fill[2]);
            assert!(bufs[2].buf[i] == fill[2]);
        }

//fill() and apply()
        for i in 0..bufs.len() {
            bufs[i].fill(fill[i]);
        }

        apply(& mut bufs, &[0,0], &[1,2], &[0,0], |l,r| l+r);

        for i in 0..BUFFER_LEN {
            assert!(bufs[0].buf[i] == fill[0] + fill[1] + fill[2]);
            assert!(bufs[1].buf[i] == fill[1]);
            assert!(bufs[2].buf[i] == fill[2]);
        }
    }
} 
