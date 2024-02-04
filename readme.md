# Functional trait

## Description

A simple macro that inspired by java's functional interface.

the macro impls a trait for Fn, FnMut or FnOnce when the trait:

- contains one and only one method

- the method has a receiver,and the receiver is `&self`, `&mut self` or `self`

- has no generics in the trait or the method (maybe I will add generics in the macro)

- has no super trait(may change in the future versions)

- is not unsafe

- have no unsafe method(may change in the future versions)

## Example

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

fn main() {
    let f = |a, b| a + b + 10;
    dbg!(f.a(1, 2));

    let mut i = 0;
    let mut f = |a, b| {
        i += 1;
        a + b + i
    };
    dbg!(f.b(1, 2));

    let s = String::new();
    let f = |a, b| {
        drop(s);
        a + b + i
    };
    dbg!(f.c(1, 2));
}

```
