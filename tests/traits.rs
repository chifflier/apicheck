use std::default::Default;

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

