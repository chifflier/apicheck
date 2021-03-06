#![feature(trait_alias)]

use std::default::Default;
use std::fmt::Debug;

pub trait Summary {
    fn summarize(&self) -> String;
}

pub trait DebugPrint : Debug {
    fn debugprint(&self);
}

pub trait Foo<T> {
    fn foo_to_string(&self, foo:T) -> String;
}



pub struct MyStruct;

impl MyStruct {
    pub fn new() -> MyStruct { MyStruct }

    fn should_not_see_me(&self) { }
}

impl Default for MyStruct {
    fn default() -> MyStruct { MyStruct }
}

// // trait aliases (see https://github.com/rust-lang/rfcs/blob/master/text/1733-trait-alias.md)
// trait SharableIterator = Iterator + Sync;
// trait IntIterator = Iterator<Item=i32>;
