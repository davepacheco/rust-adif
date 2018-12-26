//
// src/cli.rs: implementation of CLI utilities
//

use std::fs;

extern crate adif;

//
// Implementation of a simple driver command to parse a single file.
//
pub fn process_file(filename: &str) ->
    Result<(), String>
{
    let mut file = match fs::File::open(filename) {
        Ok(file) => file,
        Err(error) => {
            return Err(format!("open \"{}\": {}", filename, error))
        }
    };

    print!("{}", adif::adif_testparse_adi(filename, &mut file)?);
    Ok(())
}
