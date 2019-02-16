// #![feature(plugin)]
// #![plugin(apicheck)]

// required to test constant functions
#![feature(const_fn)]

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

// never returning function
pub fn bar() -> ! {
    panic!("bar");
}
