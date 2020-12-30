// #![feature(existential_type)]
//
// use std::fmt::Debug;
//
// // existential type in associated type position:
// struct MyType;
// impl Iterator for MyType {
//     existential type Item: Debug;
//     fn next(&mut self) -> Option<Self::Item> {
//         Some("Another item!")
//     }
// }
