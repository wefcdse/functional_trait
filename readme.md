# Functional trait

## Description

A simple macro that inspired by java's functional interface.

the macro impls a trait for [Fn], [FnMut] or [FnOnce] when the trait:

- contains one and only one method

- the method has a receiver, and the receiver is `&self`, `&mut self` or `self`

- has no generic types in the method

- is not unsafe

## Example

### use as helper trait
```rust
use functional_trait::functional_trait;
use std::future::Future;
#[functional_trait]
trait Helper1<'a> {
    fn call1(&self, s: &'a str) -> impl 'a + Future<Output = &'a str>;
}
// OR
#[functional_trait]
trait Helper2<'a> {
    type Output: Future<Output = &'a str>;
    fn call2(&self, s: &'a str) -> Self::Output;
}

async fn asyncfn(s: &str) -> &str {
    println!("{}", s);
    s
}
fn take_async1(f: impl for<'a> Helper1<'a>) {
    let string = "aaa".to_owned();
    let fut = f.call1(&string);
    // drop(string1);
    drop(fut);
    drop(string);
}
fn take_async2(f: impl for<'a> Helper2<'a>) {
    let string = "aaa".to_owned();
    let fut = f.call2(&string);
    // drop(string1);
    drop(fut);
    drop(string);
}
take_async1(asyncfn);
take_async2(asyncfn);
```

```rust
use functional_trait::functional_trait;

#[functional_trait]
trait A {
    fn a(&self, i: i32, j: i32) -> i32;
}

#[functional_trait]
trait B {
    fn b(&mut self, i: i32, j: i32) -> i32;
}

#[functional_trait]
trait C {
    fn c(self, i: i32, j: i32) -> i32;
}

#[functional_trait]
trait D {
    fn d<'c>(&self, b: &'c i32) -> &'c i32;
}

#[functional_trait]
trait E<'a, T: 'a + ?Sized, const AA: usize, T1>: Sized + Clone + Send
where
    T1: Send + Sync,
    T: std::fmt::Display,
{
    unsafe fn e<'c>(&'c self, a: &'a T, b: [i32; AA], t1: T1) -> &'a str;
}


let fa = |a, b| a + b + 10;
dbg!(fa.a(1, 2));

let mut i = 0;
let mut fb = |a, b| {
    i += 1;
    a + b + i
};
dbg!(fb.b(1, 2));

let s = String::new();
let fc = |a, b| {
    drop(s);
    a + b + i
};
dbg!(fc.c(1, 2));

let fd = {
    fn f(a: &i32) -> &i32 {
        a
    }
    f
};
fd.d(&1);

let fe = |a: &str, b: [i32; 4], _c: i128| {
    dbg!(a);
    dbg!(b);
    "413"
};
unsafe { fe.e("4fr13", [3, 5, 1, 1], 9) };

```
