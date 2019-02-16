// should not be seen, not exported
mod foo {
    pub fn foo() { }

}

pub mod bar {
    pub fn bar(a: u32) { }

    pub const AA : u32 = 0;

    fn private() { }
}

// should not be seen, not exported outside crate
pub(crate) mod baz {
    pub fn baz() { }
}

// recursive mods
pub mod a {
    pub mod b {
        pub fn inner_a_b() { }
    }

    mod c {
    }
}
