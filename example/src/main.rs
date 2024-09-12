#![allow(unused)]
use std::{future::Future, rc::Rc, sync::Arc};

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
#[functional_trait]
trait T2<'a> {
    fn a1(&self, s: &'a str) -> impl 'a + Future<Output = ()>;
}

#[functional_trait]
trait T3<'a> {
    fn a1(&self, s: &'a str) -> impl 'a + Future<Output = &'a str>;
}
// impl<'a, Fut, F> T2<'a> for F
// where
//     Fut: 'a + Future<Output = ()>,
//     F: Fn(&'a str) -> Fut,
// {
//     fn a1(&self, s: &'a str) -> impl 'a + Future<Output = ()> {
//         self(s)
//     }
// }
fn get_async2(f: impl for<'a> T2<'a>) {
    let a = String::new();

    let aa = f.a1(&a);
    // drop(a);

    drop(aa);
}

fn get_async3(f: impl for<'a> T3<'a>) {
    let a = String::new();

    let aa = f.a1(&a);
    // drop(a);

    drop(aa);
}
async fn async2(s: &str) {
    println!("{}", s);
}

async fn async3(s: &str) -> &str {
    println!("{}", s);
    s
}

fn main() {
    get_async2(async2);
    get_async3(async3);
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

// #[functional_trait]
// trait Helper {
//     type Output: Future<Output = String> + Send + Sync + 'static;
//     fn call(&self) -> &'static [Self::Output];
// }

// #[functional_trait]
// trait Helper1 {
//     type String: Future<Output = String> + Send + Sync + 'static;
//     fn call(&self) -> (Self::String, &'static [*const Self::String]);
// }

// impl<AA, F> Helper for F
// where
//     AA: Future<Output = String> + Send + Sync,
//     F: std::ops::Fn() -> AA,
// {
//     type Output = AA;
//     fn call(&self) -> Self::Output {
//         self()
//     }
// }

// impl<AA, F> Helper for F
// where
//     AA: Future<Output = String> + Send + Sync,
//     F: std::ops::Fn() -> AA,
// {
//     type Output = AA;
//     fn call(&self) -> Self::Output {
//         self()
//     }
// }

// fn take_async<'a>(f: impl 'a + Helper<'a>) {
//     let string = "aaa".to_owned();
//     // let fut = f.call(&string);
//     // drop(string1);
//     // drop(fut);
//     drop(string);
// }

fn f13f() {
    let aa = String::new();
    let f = || async {
        println!("aa");
        &aa
    };

    // take_async(f);
}
trait T4 {
    fn m4(&self, v: impl Send);
}

#[functional_trait]
trait Helper<'a> {
    type Output: Future<Output = &'a str>;
    fn call(&self, s: &'a str) -> Self::Output;
}

async fn async1(s: &str) -> &str {
    println!("{}", s);
    s
}
fn take_async(f: impl for<'a> Helper<'a>) {
    let string = "aaa".to_owned();
    let fut = f.call(&string);
    // drop(string1);
    drop(fut);
    drop(string);
}
fn aaa() {
    take_async(async1);
}
