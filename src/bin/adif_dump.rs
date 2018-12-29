//
// src/bin/adif_dump.rs: dumps contents of an ADIF file
//

use std::env;
use std::fs;
use std::process;

extern crate adif;

fn main()
{
    let argv : Vec<String> = env::args().collect();
    let progname = if argv.len() > 0 { &argv[0] } else { "adif" };

    if argv.len() > 2 {
        usage(progname, "too many arguments");
    }

    if argv.len() < 2 {
        usage(progname, "expected argument");
    }

    let filename = &argv[1];
    let which = adif::AdifDumpWhichRecords::ADR_ONE;

    match adif_dump_file(filename, which) {
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

pub fn adif_dump_file(filename: &str, which: adif::AdifDumpWhichRecords) ->
    Result<(), String>
{
    let mut file = match fs::File::open(filename) {
        Ok(file) => file,
        Err(error) => {
            return Err(format!("open \"{}\": {}", filename, error))
        }
    };

    match adif::adif_parse(filename, &mut file) {
        Ok(adif) => Ok(adif::adif_dump(adif, which)),
        Err(err) => Err(format!("{}", err))
    }
}
