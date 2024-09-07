use std::{rc::Rc, sync::Arc};

use functional_trait::functional_trait;

#[functional_trait]
trait A: Send + Sync + Sized + Clone + Unpin {
    unsafe fn aa(&'c self);
}

// impl<F: Send + Sync + Sized + Clone> A for F
// where
//     F: std::ops::Fn() -> (),
// {
//     unsafe fn aa(&self) -> () {
//         self()
//     }
// }

fn main() {
    let nonsend = Rc::new(43);
    let a = || {
        println!("Hello, world!");
        drop(nonsend);
    };
    let send = Arc::new(43);
    let a = || {
        println!("Hello, world!");
        let _ = &send;
    };
    unsafe { a.aa() };
}
