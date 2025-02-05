mod timers;
use std::{any::Any, error::Error};

use timers::{set_interval, set_timeout};

///Current state of the function. Indicates if it is being executed yet, it was rejected or it was
///completed with no errors
pub enum PromiseState<T, E> {
    Pending,
    Rejected(E),
    Done(T),
}
impl<T, E> PromiseState<T, E> {
    pub fn is_done(&self) -> bool {
        !matches!(self, Self::Pending)
    }
}
//For the ones who don't understand, dyn T means that it's any structure that implements T and we
//cannot know the size during compilation time, such as passing an interface as parameter in ts.
//Box is a pointer to the heap with some dynamic allocated object
type PromiseCb = dyn Fn(Box<dyn Any>) -> Option<Box<dyn Promise>>;
type PromiseErr = dyn Fn(Box<dyn Error>) -> Option<Box<dyn Promise>>;
trait Promise {
    ///The main function the runner is gonna call for checking
    fn poll(&mut self) -> PromiseState<Box<dyn Any>, Box<dyn Error>>;
    ///Checks if the promise has some callback when finishing
    fn chain(&self) -> Option<&PromiseCb> {
        None
    }
    ///Checks if the promise must execute some errback when erroring, if none and an error was
    ///given, it panics the thread
    fn catch(&self) -> Option<&PromiseErr> {
        None
    }
    ///Sets the given callback to be the function that is going to be executed when the promise
    ///finishes without errors
    fn then(&mut self, val: Box<PromiseCb>);
    ///Checks if the promise should be blocking the thread, if true, simply the same effect as await Promise
    ///in js
    fn should_block(&self) -> bool {
        false
    }
    ///Used to set the promise wheather the promise will block the thread or not(same effect of
    ///await in js)
    fn block(&mut self) {}
}

struct Poller {
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
    pub fn schedule<P>(&mut self, promise: P)
    where
        P: Promise + 'static,
    {
        self.in_wait.push(Box::new(promise));
    }
    #[inline]
    pub fn done(&self) -> bool {
        self.in_wait.is_empty()
    }
    ///Handles the task of the promise at index 'idx' when it completed with our without errors
    fn handle_complete(&mut self, task: PromiseState<Box<dyn Any>, Box<dyn Error>>, idx: usize) {
        let len = self.in_wait.len() - 1;
        self.in_wait.swap(idx, len); //Swaps to the last position and pops to not copy memory
        let promise = self.in_wait.pop().unwrap();
        match task {
            PromiseState::Done(val) => {
                if let Some(cb) = promise.chain() {
                    if let Some(promise) = cb(val) {
                        self.in_wait.push(promise);
                        //If the promise has a callback, calls it, and if it returns a promise,
                        //pushes it into the poller
                    };
                }
            }
            PromiseState::Rejected(err) => {
                if let Some(fb) = promise.catch() {
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
                    while promise.should_block() && !state.is_done() {
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

struct Thing {
    a: u32,
}
struct ThingProm {
    b: u32,
    cb: Option<Box<PromiseCb>>,
    should_block: bool,
}
impl Promise for ThingProm {
    fn poll(&mut self) -> PromiseState<Box<dyn Any>, Box<dyn Error>> {
        if self.b < 1000 {
            self.b += 1;
            PromiseState::Pending
        } else {
            PromiseState::Done(Box::new(Thing { a: self.b }))
        }
    }
    fn then(&mut self, val: Box<PromiseCb>) {
        self.cb = Some(val);
    }
    fn chain(&self) -> Option<&PromiseCb> {
        self.cb.as_deref()
    }
    fn should_block(&self) -> bool {
        self.should_block
    }
    fn block(&mut self) {
        self.should_block = true;
    }
}
fn f(b: u32) -> impl Promise {
    ThingProm {
        b,
        cb: None,
        should_block: false,
    }
}

fn main() {
    let mut poller = Poller::new();
    let timeout = set_interval(|| println!("run after 1 sec"), 1.0);
    let mut other = f(0);
    other.then(Box::new(|v| {
        println!("{v:?} rapaiz né que foi");
        Some(Box::new(set_timeout(
            move || println!("{v:?} depois de 2 secs"),
            2.0,
        )))
    }));
    other.block();
    let other_interval = set_interval(|| println!("After 1.5secs"), 1.5);
    poller.schedule(other);
    poller.schedule(timeout);
    poller.schedule(other_interval);
    poller.run();
}
