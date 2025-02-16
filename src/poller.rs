use std::{any::Any, error::Error};

use crate::promises::{Promise, PromiseState};

pub struct Poller {
    //List of promises in execution(must call run to initialize it)
    in_wait: Vec<Box<dyn Promise>>,
}

impl Poller {
    pub fn new() -> Self {
        Self {
            in_wait: Vec::new(),
        }
    }
    ///Adds the given promise to the poller
    #[inline]
    pub fn schedule<P: Promise + 'static>(&mut self, promise: P) {
        self.in_wait.push(Box::new(promise));
    }
    #[inline]
    pub fn done(&self) -> bool {
        self.in_wait.is_empty()
    }
    ///Handles the task of the promise at index 'idx' when it completed with our without errors
    fn handle_complete(
        &mut self,
        task: PromiseState<Option<Box<dyn Any>>, Box<dyn Error>>,
        idx: usize,
    ) {
        let len = self.in_wait.len() - 1;
        self.in_wait.swap(idx, len); //Swaps to the last position and pops to not copy memory
        let promise = self.in_wait.pop().unwrap();
        match task {
            PromiseState::Done(val) => {
                if let Some(chain) = promise.chain() {
                    if let Some(cb) = chain(val) {
                        self.in_wait.push(cb);
                    };
                }
            }
            PromiseState::Rejected(err) => {
                if let Some(fb) = promise.chain_err() {
                    if let Some(promise) = fb(err) {
                        self.in_wait.push(promise);
                        //The same description applies to here
                    }
                } else {
                    panic!("{err}");
                }
            }
            _ => {}
        }
    }
    pub fn run(&mut self) {
        while !self.done() {
            let mut idx = 0;
            while let Some(promise) = self.in_wait.get_mut(idx) {
                //Blocks the execution until the promise finishes or it not requests to block
                //anymore
                if promise.should_block() {
                    let mut state = promise.poll();
                    while !state.is_done() {
                        state = promise.poll();
                    }
                    self.handle_complete(state, idx);
                    continue;
                }
                let state = promise.poll();
                if state.is_done() {
                    self.handle_complete(state, idx);
                }
                idx += 1;
            }
        }
    }
}
