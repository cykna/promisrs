use crate::{Promise, PromiseState};
use std::{any::Any, error::Error, time::Instant};
pub struct Timeout<F>
where
    F: Fn(),
{
    f: F,
    now: Instant,
    secs: f32,
}
pub fn set_timeout<F>(f: F, secs: f32) -> Timeout<F>
where
    F: Fn(),
{
    Timeout {
        f,
        now: Instant::now(),
        secs,
    }
}
impl<F: Fn()> Promise for Timeout<F> {
    fn poll(&mut self) -> PromiseState<Box<dyn Any>, Box<dyn Error>> {
        ///Checks if the time the promise is alive is gt than the time it asks for the execution,
        ///if so, executes the callback and finishes leaving the poll tasks
        if self.now.elapsed().as_secs_f32() >= self.secs {
            (self.f)();
            PromiseState::Done(Box::new(()))
        } else {
            PromiseState::Pending
        }
    }
    fn then(&mut self, _: Box<crate::PromiseCb>) {}
}
pub struct Interval<F>
where
    F: Fn(),
{
    f: F,
    now: Instant,
    ms: f32,
}
pub fn set_interval<F>(f: F, ms: f32) -> Interval<F>
where
    F: Fn(),
{
    Interval {
        f,
        now: Instant::now(),
        ms,
    }
}
impl<F: Fn()> Promise for Interval<F> {
    //The same explanation to timeout is valid here, but interval does not finish at all to make it
    //call the callback forever
    fn poll(&mut self) -> PromiseState<Box<dyn Any>, Box<dyn Error>> {
        if self.now.elapsed().as_secs_f32() > self.ms {
            self.now = Instant::now();
            (self.f)();
        };
        PromiseState::Pending
    }
    fn then(&mut self, _: Box<crate::PromiseCb>) {}
}
