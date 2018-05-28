use std::default::Default;

pub struct MyStruct;

impl MyStruct {
    pub const VALUE0 : u32 = 0;

    pub fn new() -> MyStruct { MyStruct }
}

impl Default for MyStruct {
    fn default() -> MyStruct { MyStruct }
}

