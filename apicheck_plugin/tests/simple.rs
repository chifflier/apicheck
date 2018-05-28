#![feature(plugin)]
#![plugin(apicheck_plugin)]

// required to test constant functions
#![feature(const_fn)]

use std::fmt::Debug;

///////// type aliases

pub type Point = (u8, u8);
pub type TwoTuple<T> where T : Sized = (T,T);

///////// functions

fn lintme() { }

pub fn fun_one_arg(x: u32) {
    let _ = x;
}

pub fn fun_one_arg_with_ret(x: u32) -> u32 {
    x + 1
}

pub fn fun_generic<T>(x: T) -> T {
    x
}

pub fn fun_generic_where<T>(x: T) -> T
where T : Sized {
    x
}

// constant function
pub const fn fun_constant(x: u32) -> u32 {
    x + 1
}

// variadic
extern {
    pub fn rust_interesting_average(_: u64, ...) -> f64;
}

// unsafe
pub unsafe fn as_u8_slice(v: &[i32]) -> &[u8] {
    std::slice::from_raw_parts(v.as_ptr() as *const u8, 
                               v.len() * std::mem::size_of::<i32>())
}

// extern
pub extern fn new_i32() -> i32 { 0 }
pub extern "stdcall" fn new_i32_stdcall() -> i32 { 0 }

///////// structs

// simple struct, not exported
struct Struct01 {
    a: u32
}

// pub struct
pub struct Struct02 {
    a: u32
}

// struct with pub field
pub struct Struct03 {
    pub a: u32
}

// struct with lifetime
pub struct Struct04<'a> {
    pub s: &'a[u8]
}

// generic struct
pub struct Struct05<T> {
    pub s: Option<T>
}

// generic struct with 'where' clause
pub struct Struct06<T>
where T: Debug {
    pub s: Option<T>
}

// newtype
pub struct NewType(u32);
pub struct NewType2(pub u32);
pub struct NewType3(u32, u64);
pub struct NewType4((u32, u64));

// unit
pub struct StructUnit;

///////// enums

// simple snum
pub enum Enum01 {
    Blah
}

// simple snum
pub enum Enum02 {
    Foo,
    Bar(u32),
    Baz(u32, String)
}

// enum with generics
pub enum Enum03<T>
where T: Sized {
    Foo,
    Bar(Option<T>),
    Baz
}

///////// unions

// C-like union
pub union Union01 {
    blah: u32
}
