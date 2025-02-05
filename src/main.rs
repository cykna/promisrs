mod timers;
use std::any::Any;

use timers::{set_interval, set_timeout};

pub enum PromiseState<T> {
    Pending,
    Done(T),
}
type PromiseCb = dyn Fn(Box<dyn Any>) -> Option<Box<dyn Promise>>;
trait Promise {
    fn poll(&mut self) -> PromiseState<Box<dyn Any>>;
    fn chain(&self) -> Option<&PromiseCb> {
        None
    }
    fn then(&mut self, val: Box<PromiseCb>);
}

struct Poller {
    in_wait: Vec<Box<dyn Promise>>,
}

impl Poller {
    pub fn new() -> Self {
        Self {
            in_wait: Vec::new(),
        }
    }
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
    pub fn run(&mut self) {
        while !self.done() {
            let mut idx = 0;
            while let Some(promise) = self.in_wait.get_mut(idx) {
                if let PromiseState::Done(v) = promise.poll() {
                    let len = self.in_wait.len() - 1;
                    self.in_wait.swap(idx, len);
                    let p = self.in_wait.pop().unwrap();
                    if let Some(callback) = p.chain() {
                        if let Some(val) = callback(v) {
                            self.in_wait.push(val);
                        }
                    }
                } else {
                    idx += 1;
                }
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
}
impl Promise for ThingProm {
    fn poll(&mut self) -> PromiseState<Box<dyn Any>> {
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
}
fn f(b: u32) -> impl Promise {
    ThingProm { b, cb: None }
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
    poller.schedule(timeout);
    poller.schedule(other);
    poller.run();
}
