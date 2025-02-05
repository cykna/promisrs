use crate::{Promise, PromiseState};
use std::{any::Any, time::Instant};
pub struct Timeout<F>
where
    F: Fn(),
{
    f: F,
    now: Instant,
    ms: f32,
}
pub fn set_timeout<F>(f: F, ms: f32) -> Timeout<F>
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
    fn poll(&mut self) -> PromiseState<Box<dyn Any>> {
        if self.now.elapsed().as_secs_f32() > self.ms {
            self.now = Instant::now();
            (self.f)();
        };
        PromiseState::Pending
    }
}
