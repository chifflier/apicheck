#[derive(Debug, PartialEq)]
pub struct MyStruct;

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Struct01 {
    pub a: u32,
}

#[test]
pub fn test_foo() {
    /* ... */
}

#[cfg(target_os = "linux")]
pub mod bar {
    /* ... */
}

// A lint attribute used to suppress a warning/error
#[allow(non_camel_case_types)]
pub type int8_t = i8;

