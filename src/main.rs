//
// src/main.rs: driver program for ADIF parser.  For now, this just parses the
// given ADI file and dumps the result.
//

use std::env;
use std::process;

extern crate adif;
mod cli;

fn main() {
    let argv : Vec<String> = env::args().collect();
    let progname = if argv.len() > 0 { &argv[0] } else { "adif" };

    if argv.len() > 2 {
        usage(progname, &"too many arguments");
    }

    if argv.len() < 2 {
        usage(progname, &"expected argument");
    }

    let filename = &argv[1];
    match cli::process_file(filename) {
        Ok(()) => (),
        Err(errmsg) => fatal(progname, &errmsg)
    }
}

fn usage(progname: &str, message: &str)
{
    eprintln!("{}", message);
    eprintln!("usage: {} FILENAME", progname);
    process::exit(2);
}

fn fatal(progname: &str, message: &str)
{
    eprintln!("{}: {}", progname, message);
    process::exit(1);
}
