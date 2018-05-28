extern crate libapicheck;

use std::env;

fn main() {
    for argument in env::args().skip(1) {
        println!("arg: {}", argument);
        libapicheck::process_file(argument);
    }
}
