use std::{any::Any, error::Error};

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
pub type PromiseCb = dyn Fn(Option<Box<dyn Any>>) -> Option<Box<dyn Promise>>;
pub type PromiseErr = dyn Fn(Box<dyn Error>) -> Option<Box<dyn Promise>>;
pub trait Promise {
    ///The main function the runner is gonna call for checking
    fn poll(&mut self) -> PromiseState<Option<Box<dyn Any>>, Box<dyn Error>>;
    ///Checks if the promise has some callback when finishing
    fn chain(&self) -> Option<&PromiseCb> {
        None
    }
    fn chain_err(&self) -> Option<&PromiseErr> {
        None
    }
    ///Checks if the promise must execute some errback when erroring, if none and an error was
    ///given, it panics the thread
    fn catch(&mut self, _: Box<PromiseErr>) {}
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
pub struct GenericPromise {
    chain: Option<Box<PromiseCb>>,
    error: Option<Box<PromiseErr>>,
    f: Box<dyn FnMut(Option<&mut dyn Any>) -> PromiseState<Option<Box<dyn Any>>, Box<dyn Error>>>,
    blocking: bool,
    data: Option<Box<dyn Any>>,
}
pub fn promise<F>(f: F, data: Option<Box<dyn Any>>) -> GenericPromise
where
    F: (FnMut(Option<&mut dyn Any>) -> PromiseState<Option<Box<dyn Any>>, Box<dyn Error>>)
        + 'static,
{
    GenericPromise {
        chain: None,
        error: None,
        f: Box::new(f),
        blocking: false,
        data,
    }
}
impl Promise for GenericPromise {
    fn poll(&mut self) -> PromiseState<Option<Box<dyn Any>>, Box<dyn Error>> {
        (self.f)(self.data.as_deref_mut())
    }
    fn then(&mut self, val: Box<PromiseCb>) {
        self.chain = Some(val);
    }
    fn catch(&mut self, val: Box<PromiseErr>) {
        self.error = Some(val);
    }
    fn chain_err(&self) -> Option<&PromiseErr> {
        self.error.as_deref()
    }
    fn should_block(&self) -> bool {
        self.blocking
    }
    fn block(&mut self) {
        self.blocking = true;
    }
}
