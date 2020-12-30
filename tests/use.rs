pub mod foo {
    pub fn fn_foo() {}
    pub fn fn_bar() {}
    pub fn fn_baz() {}

    pub mod submod {
        pub fn fn_sub1() {}
    }
}

pub mod mod1 {
    pub use foo;
}
pub mod mod2 {
    pub use foo::fn_foo;
}
pub mod mod3 {
    pub use foo::{fn_bar, fn_foo};
}
pub mod mod4 {
    pub use foo::*;
}
pub mod mod5 {
    pub use foo as blah;
}
pub mod mod6 {
    pub use foo::{fn_foo, submod::fn_sub1};
}
