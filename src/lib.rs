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

mod adi;

const ADIF_HEADER_ADIF_VER : &'static str = "adif_ver";
const ADIF_HEADER_CREATED_TIMESTAMP : &'static str = "created_timestamp";
const ADIF_HEADER_PROGRAMID : &'static str = "programid";
const ADIF_HEADER_PROGRAMVERSION : &'static str = "programversion";
const ADIF_HEADER_USERDEF : &'static str = "userdef";

pub fn adif_testparse(source : &mut io::Read) -> Result<String, String>
{
    // TODO flesh out
    match adi::adi_parse(source) {
        Ok(r) => Ok(format!("{}", adi::adi_dump(r))),
        Err(e) => Err(format!("error: {:?}", e))
    }
}

pub fn adif_testparse_string(source : &str) -> Result<String, String>
{
    // TODO should remove adi_parse_string() and do that work here instead?
    match adi::adi_parse_string(source) {
        Ok(r) => Ok(format!("{}", adi::adi_dump(r))),
        Err(e) => Err(format!("error: {:?}", e))
    }
}
