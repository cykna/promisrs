mod poller;
mod promises;
mod timers;

use std::{cell::RefCell, rc::Rc};

use poller::Poller;
use promises::PromiseState;
use timers::{set_interval, set_timeout};

fn main() {
    let mut poller = Poller::new();
    let n = Rc::new(RefCell::new(5));
    let m = Rc::clone(&n);
    let data = promises::promise(
        move |mut val| {
            let m = val.unwrap().downcast_mut::<Rc<RefCell<i32>>>().unwrap();
            let mut m = m.borrow_mut();
            println!("{}", *m);
            if *m < 10000000 {
                *m += 1;
                PromiseState::Pending
            } else {
                PromiseState::Done(None)
            }
        },
        Some(Box::new(Rc::clone(&n))),
    );
    let timeout = set_interval(
        move || {
            let mut m = n.borrow_mut();
            *m += 1;
            println!("run after 1 sec; increased n in interval 1 N = {}", *m)
        },
        0.01,
    );
    let interval = set_interval(
        move || {
            println!(
                "After 1.5secs; increased n in interval 2 N = {}",
                *m.borrow()
            );
        },
        0.01,
    );
    poller.schedule(data);
    poller.schedule(timeout);
    poller.schedule(interval);
    poller.run();
}
