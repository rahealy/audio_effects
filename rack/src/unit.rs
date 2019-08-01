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
///Unit contains effects processors. provides interfaces to the 
///acyclical graph formed by the connections between processors 
///including processing and dispatching data throughout the graph.
///

use shared::block::{Buffers, Connectors, Input, Output};
use shared::processor::{Processor};
use shared::connector::{Connector, Connection, EndPoint};
use shared::buffer::{Write};
use std::collections::vec_deque::VecDeque;
use std::ops::IndexMut;

/**********************************************************************
 * get_refs()
 *********************************************************************/

///
///Rust-centric way to get two references to two items in the same 
///slice. Huh.
///
fn get_refs<T>(slice: &mut [T], 
               idx1: usize, 
               idx2: usize) -> (&mut T, &mut T) 
{
    if idx1 < idx2 {
        let (beg, end) = slice.split_at_mut(idx2);
        (&mut beg[idx1], &mut end[0])
    } else {
        let (beg, end) = slice.split_at_mut(idx1);
        (&mut end[0], &mut beg[idx2])
    }
}


/**********************************************************************
 * Dispatch
 *********************************************************************/

///
///Contains a list of outputs from a single processor that need to be
///dispatched to the inputs of other connected processors.
///
#[derive(Default)]
struct Dispatch {
    cons: Vec::<Connection>,
    proc: usize
}


/**********************************************************************
 * State
 *********************************************************************/

enum State {
    Started,
    Stopped
}

impl Default for State {
    fn default() -> State {
        State::Stopped
    }
}

/**********************************************************************
 * Unit
 *********************************************************************/

///
///Provides an interface for making and breaking connections from
///processor outputs to processor inputs. Dispatches data throughout
///the acyclical graph formed by the same.
///
#[derive(Default)]
pub struct Unit<'a> {
    procs:    Vec<&'a mut dyn Processor>, //Stores all processors.
    next:     VecDeque<usize>,            //Next processor to process. FIFO.
    forward:  VecDeque<Dispatch>,         //Dispatches forward FIFO.
    backward: VecDeque<Dispatch>,         //Dispatches backward FIFO.
    start:    Vec<usize>,                 //Start nodes in connection graph.
    state:    State
}


impl <'a> Unit<'a> {
    fn print_proc_msg(&self, msg: &'static str, p_idx: usize) -> () {
//         println!(
//             "{} ({}) {}",
//             msg,
//             p_idx,
//             self.procs[p_idx].info().name
//         );
    }

///
/// Process a buffer's worth of work in the currently queued processor.
///
    pub fn process_next(&mut self) -> () {
        if let Some(p_idx) = self.next.pop_front() {
            self.print_proc_msg("unit::process_next(): Processing", p_idx);

            let mut proc =  &mut self.procs[p_idx];
            let mut disp = Dispatch::default();            

//Process and gather output connections to dispatch forward.
            proc.process();
            proc.map_outputs (
                &mut |o_blk| {
                    for conn in o_blk.connectors().iter() {
                        if let Connector::ConnectedUsing(con) = conn {
                            disp.cons.push(*con); 
                        }
                    }
                    true
                }
            );

//Queue dispatch.
            disp.proc = p_idx;
            self.forward.push_back(disp);
        }
    }


///
///Send the output of the currently queued dispatch to the inputs of
///the receiving processors. Queue receiving processors whose inputs
///are all full to process in the unit's 'next' list. Queue the
///backward dispatches for each connection.
///
    pub fn dispatch_next_forward(&mut self) -> () {
        if let Some(d) = self.forward.pop_front() {
            self.print_proc_msg("unit::dispatch_next_forward(): Dispatching from", d.proc);

//Update all the connections in the dispatch.
            for con in d.cons.iter() {
                let (p_from, p_to) = get_refs(&mut self.procs, 
                                            con.from.proc, 
                                            con.to.proc);
//Copy from output to input.
                p_to.input(con.to.block)
                    .buffer(con.to.conn)
                    .copy_from(&p_from.output(con.from.block)
                                    .buffer(con.from.conn));

//Reset output buffer so it can be written to again.
                p_from.output(con.from.block)
                    .buffer(con.from.conn)
                    .reset();

//Input buffer in block has been filled from the output buffer.
                p_to.input(con.to.block).inc_full_cnt();

//Output buffer in block has drained into the input buffer.
                p_from.output(con.from.block).inc_empty_cnt();

                if p_to.map_inputs( &mut |blk| { blk.full_cnt() == blk.num_cons() } ) {
//All inputs are full.
                    p_to.map_inputs ( //Reset full counters.
                        &mut |blk| { 
                            blk.rst_full_cnt(); 
                            return true;
                        } 
                    );

//Queue processor.
                    self.print_proc_msg("unit::dispatch_one_forward(): Queueing", con.to.proc);
                    self.next.push_back(con.to.proc);

//Queue backward dispatch.
                    self.backward.push_back (
                        Unit::new_back_dispatch(&mut self.procs, con.to.proc)
                    );
                }
            }
        }
    }


    pub fn dispatch_backward(&mut self) -> () {
        for dspch in self.backward.drain(..) {
            for con in dspch.cons.iter() { 
                let proc = &mut self.procs[con.from.proc];

                if proc.map_outputs ( &mut |blk| { blk.empty_cnt() == blk.num_cons() } ) {
//All outputs are empty. Reset empty counters.
                    proc.map_outputs ( 
                        &mut |blk| { 
                            blk.rst_empty_cnt(); 
                            return true; 
                        }
                    );

                    if let Some(_) = self.start
                                         .iter()
                                         .position(|&x| x == con.from.proc) 
                    {
                        self.next.push_back(con.from.proc);
                    }
                }
            }
        }
    }


    fn new_back_dispatch(
        slice: &mut [&mut dyn Processor], 
        p_fwd_idx: usize) -> Dispatch 
    {
//Gather unique indexes of all processors with one or more outputs 
//connected to the forward processor.
        let mut bk_procs = Vec::<usize>::default();

        if let Some(fwd_proc) = slice.get_mut(p_fwd_idx) {
            fwd_proc.map_inputs (
                &mut |fwd_blk: &mut Input| {
                    for fwd_conn in fwd_blk.connectors().iter() {
                        if let Connector::ConnectedUsing(fwd_con) = fwd_conn {
                            if let None = bk_procs.iter()
                                                  .position(|&x| x == fwd_con.to.proc)
                            {
                                bk_procs.push(fwd_con.to.proc);
                            }
                        }
                    }
                    true
                }
            );
        }

//Gather connections of all processor output buffers.
        let mut disp = Dispatch::default();

        for p_bk_idx in bk_procs.iter() {
            if let Some(bk_proc) = slice.get_mut(*p_bk_idx) {
                bk_proc.map_outputs (
                    &mut |bk_blk: &mut Output| {
                        for bk_conn in bk_blk.connectors().iter() {
                            if let Connector::ConnectedUsing(bk_con) = bk_conn {
                                disp.cons.push(*bk_con);
                            }
                        }
                        true
                    }
                );
            }
        }

        disp.proc = p_fwd_idx;
        return disp;
    }

///
///Determine if a processor should be in the start list or not. Add/remove
///processor from the start list as necessary.
///
    fn update_start_list(&mut self, p_idx: usize) {
        let mut add_flg = true;

        self.procs[p_idx].map_inputs (
            &mut |i_blk| {
                if i_blk.b.num_cons > 0 { //Processor is not a start node.
                    add_flg = false;
                    false //Break.
                } else {
                    true  //Continue.
                }
            }
        );

        if add_flg {
            if let None = self.start
                              .iter()
                              .position(|&x| x == p_idx) 
            {
                self.print_proc_msg ("update_start_list(): Adding processor", p_idx);
                self.start.push(p_idx);
                self.next.push_back(p_idx);
            }
        } else {
            if let Some(s_idx) = self.start
                                     .iter()
                                     .position(|&x| x == p_idx) 
            {
                self.print_proc_msg ("update_start_list(): Removing processor", p_idx);
                self.start.remove(s_idx);

                if let Some(n_idx) = self.next
                                         .iter()
                                         .position(|&x| x == p_idx) {
                    self.next.remove(n_idx);
                }
            }
        }
    }


///
/// Make a connection from the output of one processor in the unit to
/// the input of another processor in the unit.
///
    pub fn connect(&mut self, con: Connection) -> Result<(), &'static str> {
        if self.started() {
            return Err("Unit::connect(): Can not make connections while started.");
        }

        let (p_from, p_to) = get_refs(&mut self.procs, con.from.proc, con.to.proc);

        p_from.output(con.from.block)
              .connect(Connection {from: con.from, to: con.to})?;

        if let Err(e) = p_to.input(con.to.block)
                            .connect(Connection{from: con.to, to: con.from}) 
        {
            if let Err(_) = p_from.output(con.from.block)
                                  .disconnect(con.from.conn) {
                panic!("unit.connect(): This should never error!");
            }

            return Err(e);
        }

        self.update_start_list(con.to.proc);

        return Ok(());
    }


///
/// Break a connection from the output of one processor in the unit to
/// the input of another processor in the unit.
///
    pub fn disconnect(&mut self, con: Connection) -> Result<(), &'static str> {
        if self.started() {
            return Err("Unit::connect(): Can not break connections while started.");
        }

        if self.connection_exists(con) {
            let (p_from, p_to) = get_refs(&mut self.procs, con.from.proc, con.to.proc);
            
            p_from.output(con.from.block).disconnect(con.from.conn)?;
            p_to.input(con.to.block).disconnect(con.to.conn)?;
            self.update_start_list(con.to.proc);

            Ok(())
        } else {
            Err("unit.disconnect(): Connection doesn't exist.")
        }
    }


///
/// Determines if a connection from the output of one processor in the 
/// unit to the input of another processor in the unit exists.
///
    fn connection_exists(&mut self, con: Connection) -> bool {
        let (p_from, p_to) = get_refs(&mut self.procs, con.from.proc, con.to.proc);
        
        if let Connector::ConnectedUsing(con_from) = p_from.output(con.from.block)
                                                           .connector(con.from.conn) 
        {
            if (con_from.from == con.from) && (con_from.to == con.to) {
                if let Connector::ConnectedUsing(con_to) = p_to.input(con.to.block)
                                                               .connector(con.to.conn)
                {
                    return (con_to.from == con.to) && (con_to.to == con.from);
                }
            }
        }

        return false;
    }


///
/// Add a processor to the unit.
///
    pub fn add(&mut self, proc: &'a mut dyn Processor) -> Result<(), &'static str> {
        if self.started() {
            return Err("Unit::add(): Can not add processors while started.");
        }

        self.start.push(self.procs.len());
        self.procs.push(proc);
        
        Ok(())
    }

///
/// Return number of processors in list.
///
    pub fn num_processors(&self) -> usize {
        self.procs.len()
    }

///
/// Access processor at position.
///
    pub fn processor(&mut self, idx: usize) -> &mut dyn Processor {
        if let Some(x) = self.procs.get_mut(idx) {
            *x
        } else {
            panic!("Index out of bounds.");
        }
    }

///
///Prepare the unit to process.
///
    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.started() {
            return Err("Unit::start(): Already started.");
        }

        if self.next.is_empty() {
            for i in self.start.iter() {
                self.next.push_back(*i);
            }
        }

        self.state = State::Started;

        Ok(())
    }


///
///Drain all the current processing queues and stop.
///
    pub fn drain_and_stop(&mut self) -> Result<(), &'static str> {
        if !self.started() {
            return Err("Unit::drain_and_stop(): Already stopped.");
        }
        
        self.dispatch_backward();

        while !self.next.is_empty() {
            self.process_next();
            self.dispatch_next_forward();
        }

        self.state = State::Stopped;
        Ok(())
    }

    fn started(&self) -> bool {
        match self.state {
            State::Started => true,  
            State::Stopped => false   
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn unit() {
//FIXME: This is a time consuming job which needs to be done.
    }
}
