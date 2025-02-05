use std::{any::Any, time::Instant};

pub enum PromiseState<T> {
    Pending,
    Done(T),
}
impl<T: Any> PromiseState<T> {
    fn done(val: T) -> Self {
        Self::Done(val)
    }
}
trait Promise {
    fn poll(&mut self) -> PromiseState<Box<dyn Any>>;
}

struct Poller<'a> {
    in_wait: Vec<&'a mut dyn Promise>,
}

impl<'a> Poller<'a> {
    pub fn new() -> Self {
        Self {
            in_wait: Vec::new(),
        }
    }
    #[inline]
    pub fn schedule<P>(&mut self, promise: &'a mut P)
    where
        P: Promise,
    {
        self.in_wait.push(promise);
    }
    #[inline]
    pub fn done(&self) -> bool {
        self.in_wait.is_empty()
    }
    pub fn run(&mut self) {
        'lp: loop {
            if self.done() {
                break;
            }
            let mut idx = 0;
            while let Some(promise) = self.in_wait.get_mut(idx) {
                if let PromiseState::Done(v) = promise.poll() {
                    let len = self.in_wait.len() - 1;
                    self.in_wait.swap(idx, len);
                    self.in_wait.pop();
                } else {
                    idx += 1;
                }
            }
        }
    }
}

struct Timeout<F>
where
    F: Fn(),
{
    f: F,
    now: Instant,
    ms: f32,
}
fn set_timeout<F>(f: F, ms: f32) -> Timeout<F>
where
    F: Fn(),
{
    Timeout {
        f,
        now: Instant::now(),
        ms,
    }
}
impl<F: Fn()> Promise for Timeout<F> {
    fn poll(&mut self) -> PromiseState<Box<dyn Any>> {
        if self.now.elapsed().as_secs_f32() >= self.ms {
            (self.f)();
            PromiseState::Done(Box::new(()))
        } else {
            PromiseState::Pending
        }
    }
}
fn main() {
    let mut poller = Poller::new();
    let mut timeout = set_timeout(|| println!("run after 1 sec"), 1.0);
    poller.schedule(&mut timeout);
    poller.run();
}
