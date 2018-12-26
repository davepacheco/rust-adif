//
// Amateur Data Interchange Format (ADIF) is a standardized file format used for
// exchanging data about amateur radio contacts ("QSOs").  This crate seeks to
// implement an ADIF importer and a small reporting program.
//
// As of this writing, the latest ADIF standard is v3.0.8, available here:
//
//   http://www.adif.org/308/ADIF_308.htm
//
// Note that much of ADIF describes a logical form for the data.  There are
// currently two physical file formats: ADI (a somewhat baroque format described
// originally in version 1, which dates back to 1996) and ADX (a more modern
// XML-based format).  ADI appears to be more widely used, while ADX is marked
// optional in the standard.  For that reason, this crate currently only seeks
// to implement ADI.
//
// Section II.A ("Upward Compatibility") guarantees that "an ADIF file compliant
// with ADIF version N will comply with any future ADIF version M where M>N."
// By implementing v3, we support all v1 and v2 files.
//

use std::io;
use std::fmt;

mod adi;
mod adif;
mod adifutil;

//
// AdifParseError is used to represent any sort of operational error we may
// encounter during parsing.
//

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum AdifParseError {
    ADIF_EIO(io::Error),                  // error from underlying I/O
    ADIF_EBADINPUT(String),               // invalid input
    ADIF_ENOT_YET_IMPLEMENTED(String),    // feature that's not yet implemented
}

impl From<io::Error> for AdifParseError {
    fn from(error: io::Error) -> Self {
        AdifParseError::ADIF_EIO(error)
    }
}

impl fmt::Display for AdifParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdifParseError::ADIF_EIO(ioerror) => {
                write!(f, "{}", ioerror.to_string())
            },
            AdifParseError::ADIF_EBADINPUT(message) => {
                write!(f, "input error: {}", message)
            },
            AdifParseError::ADIF_ENOT_YET_IMPLEMENTED(message) => {
                write!(f, "not yet implemented: {}", message)
            }
        }
    }
}

//
// These entry points are provided for now just for testing.
//
pub fn adif_testparse_adi(label: &str, source : &mut io::Read) ->
    Result<String, String>
{
    // TODO flesh out
    let mut rv = String::new();
    match adi::adi_parse(source) {
        Ok(r) => {
            rv.push_str(&format!("{}", adi::adi_dump(&r)));
            rv.push_str("\n\n");
            match adif::adif_parse_adi(label, &r) {
                Ok(adif) => {
                    rv.push_str(&format!("{:?}\n", adif));
                }
                Err(e) => {
                    rv.push_str(&format!("{}", e));
                }
            }
        }
        Err(e) => {
            rv.push_str(&format!("{}", e));
            return Err(rv);
        }
    }

    Ok(rv)
}

pub fn adif_testparse_adi_string(source : &str) -> Result<String, String>
{
    // TODO should remove adi_parse_string() and do that work here instead?
    match adi::adi_parse_string(source) {
        Ok(r) => Ok(format!("{}", adi::adi_dump(&r))),
        Err(e) => Err(format!("{}", e))
    }
}
