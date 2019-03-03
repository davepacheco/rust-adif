//
// src/bin/adif_diff.rs: dumps differences between two ADIF files
// This is currently very primitive.  It doesn't sort the QSOs, it doesn't find records in the
// second file that aren't in the first, it only matches records up using only the date and
// callsign, and it only compares the grid square.
//

use std::env;
use std::fs;
use std::io;
use std::process;

extern crate adif;

fn main()
{
    let argv : Vec<String> = env::args().collect();
    let progname = if argv.len() > 0 { &argv[0] } else { "adif_diff" };

    if argv.len() != 3 {
        usage(progname, "expected two arguments");
    }

    let fname1 = &argv[1];
    let fname2 = &argv[2];

    match adif_diff_files(fname1, fname2) {
        Ok(()) => (),
        Err(errmsg) => fatal(progname, &errmsg)
    }
}

fn usage(progname: &str, message: &str)
{
    eprintln!("{}", message);
    eprintln!("usage: {} FILENAME1 FILENAME2", progname);
    process::exit(2);
}

fn fatal(progname: &str, message: &str)
{
    eprintln!("{}: {}", progname, message);
    process::exit(1);
}

fn open_file(filename: &str) ->
    Result<fs::File, String>
{
    match fs::File::open(filename) {
        Ok(file) => Ok(file),
        Err(error) => {
            return Err(format!("open \"{}\": {}", filename, error))
        }
    }
}

fn adif_diff_files(fname1: &str, fname2: &str) ->
    Result<(), String>
{
    let mut f1 = open_file(fname1)?;
    let mut f2 = open_file(fname2)?;

    adif_diff_streams(fname1, &mut f1, fname2, &mut f2)
}

fn make_qso_sig(record : &adif::AdifRecord) ->
    String
{
    // XXX should use time, too, but for the logs I care about, the fields are slightly
    // inconsistent, so it needs to be a fuzzy match.
    format!("{} QSO with {}",
        record.adir_field_values["qso_date"],
        record.adir_field_values["call"])
}

fn adif_diff_streams(label1 : &str, source1 : &mut io::Read,
    label2 : &str, source2 : &mut io::Read) ->
    Result<(), String>
{
    let adf1 = match adif::adif_parse(label1, source1) {
        Ok(adf) => adf,
        Err(error) => return Err(format!("{}", error))
    };

    let adf2 = match adif::adif_parse(label2, source2) {
        Ok(adf) => adf,
        Err(error) => return Err(format!("{}", error))
    };

    let mut nmatched = 0;
    let mut nunmatched1 = 0;
    let mut ndiff = 0;

    let l1 = adf1.adif_records.len();
    let l2 = adf2.adif_records.len();

    for i in 0..l1 {
        let r1 = &adf1.adif_records[i];
        let sig1 = make_qso_sig(&r1);
        let mut found = None;

        // XXX awful complexity
        for j in 0..l2 {
            let r2 = &adf2.adif_records[j];
            let sig2 = make_qso_sig(&r2);
            if sig1 == sig2 {
                // XXX should tag record so it's not re-used
                found = Some(r2);
                break;
            }
        }

        if let None = found {
            nunmatched1 += 1;
            println!("only in {}: {}", label1, sig1);
            continue;
        }

        let r2 = found.unwrap();
        nmatched += 1;

        if !r1.adir_field_values.contains_key("gridsquare") {
            if !r2.adir_field_values.contains_key("gridsquare") {
                continue;
            }

            ndiff += 1;
            println!("grid squares differ: {} (none vs. \"{}\")",
                sig1, r2.adir_field_values["gridsquare"]);
            continue;
        }

        if !r2.adir_field_values.contains_key("gridsquare") {
            ndiff += 1;
            println!("grid squares differ: {} (\"{}\" vs. none)",
                sig1, r1.adir_field_values["gridsquare"]);
            continue;
        }

        if r1.adir_field_values["gridsquare"] !=
           r2.adir_field_values["gridsquare"] {
            ndiff += 1;
            println!("grid squares differ: {} (\"{}\" vs. \"{}\")",
                sig1, r1.adir_field_values["gridsquare"],
                r2.adir_field_values["gridsquare"]);
        }
    }

    println!("records only in {}: {}", label1, nunmatched1);
    println!("matched records: {}", nmatched);
    println!("matched records with differences: {}", ndiff);

    // XXX should go through un-tagged records
    Ok(())
}
