extern crate getopts;

extern crate libapicheck;

use getopts::Options;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // process options
    let mut opts = Options::new();
    opts.optflagmulti("d", "debug", "display debug information");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&args[0], opts);
        return;
    }
    // setup config
    let mut config = libapicheck::config::Config::default();
    if matches.opt_present("d") { config.debug = true; }
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&args[0], opts);
        return;
    };
    // work !
    if config.debug { println!("Processing file {}", input); }
    libapicheck::process_file(input, &config);
}
