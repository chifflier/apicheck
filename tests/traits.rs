use std::default::Default;

pub trait Summary {
    fn summarize(&self) -> String;
}

pub trait DebugPrint : Debug {
    fn debugprint(&self);
}

pub struct MyStruct;


