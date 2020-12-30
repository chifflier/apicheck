extern crate getopts;

extern crate libapicheck;

use getopts::Options;
use std::env;
use std::path::PathBuf;

use libapicheck::config::FileName;
use libapicheck::Input;

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
    opts.optopt("o", "output", "output file name", "FILE");
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
    config.debug = matches.opt_count("d");
    config.output = match matches.opt_str("o") {
        Some(s) => {
            if &s == "-" { FileName::Stdout }
            else { FileName::Real(PathBuf::from(s.clone())) }
        },
        None    => FileName::Stdout,
    };
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&args[0], opts);
        return;
    };
    // work !
    if config.debug > 0 { println!("Processing file {}", input); }
    let input = Input::File(PathBuf::from(input));
    libapicheck::process_file(input, &config);
}
