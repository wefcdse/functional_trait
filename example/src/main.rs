use std::{rc::Rc, sync::Arc};

use functional_trait::functional_trait;

#[functional_trait]
trait A: Send + Sync + Sized + Clone + Unpin {
    unsafe fn aa(&self);
}

#[functional_trait]
trait E<'a, T: 'a + ?Sized, const AA: usize, T1>: Sized + Clone + Send
where
    T1: Send,
{
    unsafe fn e<'c>(&'c self, a: &'a T, b: [i32; AA], t1: T1) -> &'a str;
}

// impl<F: Send + Sync + Sized + Clone> A for F
// where
//     F: std::ops::Fn() -> (),
// {
//     unsafe fn aa(&self) -> () {
//         self()
//     }
// }

trait T1<'a, T> {
    fn tt();
}

impl<'a, T, F> T1<'a, T> for F
where
    F: std::ops::Fn() -> (),
{
    fn tt() {
        todo!()
    }
}

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
    let e = |a: &str, b: [i32; 4], _c: i128| {
        dbg!(a);
        dbg!(b);
        "413"
    };
    unsafe { e.e("4fr13", [3, 5, 1, 1], 9) };
    unsafe { a.aa() };
}
