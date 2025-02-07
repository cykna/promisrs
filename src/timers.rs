use std::{any::Any, error::Error, time::Instant};

use crate::promises::{Promise, PromiseState};
pub struct Timeout<F>
where
    F: FnMut(),
{
    f: F,
    now: Instant,
    secs: f32,
}
pub fn set_timeout<F>(f: F, secs: f32) -> Timeout<F>
where
    F: FnMut(),
{
    Timeout {
        f,
        now: Instant::now(),
        secs,
    }
}
impl<F: FnMut()> Promise for Timeout<F> {
    fn poll(&mut self) -> PromiseState<Option<Box<dyn Any>>, Box<dyn Error>> {
        //Checks if the time the promise is alive is gt than the time it asks for the execution,
        //if so, executes the callback and finishes leaving the poll tasks
        if self.now.elapsed().as_secs_f32() >= self.secs {
            (self.f)();
            PromiseState::Done(None)
        } else {
            PromiseState::Pending
        }
    }
    fn then(&mut self, _: Box<crate::promises::PromiseCb>) {}
}
pub struct Interval<F>
where
    F: FnMut(),
{
    f: F,
    now: Instant,
    ms: f32,
}
pub fn set_interval<F>(f: F, ms: f32) -> Interval<F>
where
    F: FnMut(),
{
    Interval {
        f,
        now: Instant::now(),
        ms,
    }
}
impl<F: FnMut()> Promise for Interval<F> {
    //The same explanation to timeout is valid here, but interval does not finish at all to make it
    //call the callback forever
    fn poll(&mut self) -> PromiseState<Option<Box<dyn Any>>, Box<dyn Error>> {
        if self.now.elapsed().as_secs_f32() > self.ms {
            self.now = Instant::now();
            (self.f)();
        };
        PromiseState::Pending
    }
    fn then(&mut self, _: Box<crate::promises::PromiseCb>) {}
}
