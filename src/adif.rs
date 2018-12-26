//
// src/adif.rs: logical ADIF parser
//
// The facilities in this file convert the physical representation of an ADI
// file into a more useful interface for consumers.
//

use adi::AdiFile;
use adi::AdiDataSpecifier;
use super::AdifParseError;

// Well-known header fields
const ADIF_HEADER_ADIF_VER : &'static str = "adif_ver";
const ADIF_HEADER_CREATED_TIMESTAMP : &'static str = "created_timestamp";
const ADIF_HEADER_PROGRAMID : &'static str = "programid";
const ADIF_HEADER_PROGRAMVERSION : &'static str = "programversion";
const ADIF_HEADER_USERDEF : &'static str = "userdef";

#[derive(Debug)]
pub struct AdifFile {
    // Well-known header fields
    pub adif_adif_version : Option<String>,     // XXX semver type?
    pub adif_program_id : Option<String>,
    pub adif_program_version : Option<String>,
    pub adif_created_timestamp : Option<String>,    // XXX date type

    // Metadata
    pub adif_label : String,    // label for this file (e.g., filename)

    // XXX map of application-defined field metadata and values?
    // XXX map of user-defined field metadata and values?

    // File contents
    pub adif_records : Vec<AdifRecord>,     // list of records in the file
}

#[derive(Debug)]
pub struct AdifRecord {
}

// TODO Would this be better off accepting an iterator?
pub fn adif_parse(label: &str, adi: &AdiFile) ->
    Result<AdifFile, AdifParseError>
{
    let mut adif = AdifFile {
        adif_adif_version: None,
        adif_program_id: None,
        adif_program_version: None,
        adif_created_timestamp: None,
        adif_label: String::from(label), // XXX clone needed?
        adif_records: Vec::with_capacity(adi.adi_records.len()),
    };

    if let Some(ref adih) = adi.adi_header {
        // TODO can this be made table-based?
        for adf in &adih.adih_fields {
            if adf.adif_name_canon == ADIF_HEADER_ADIF_VER {
                adif.adif_adif_version = Some(adif_string(&adf)?);
            } else if adf.adif_name_canon == ADIF_HEADER_PROGRAMID {
                adif.adif_program_id = Some(adif_string(&adf)?);
            } else if adf.adif_name_canon == ADIF_HEADER_PROGRAMVERSION {
                adif.adif_program_version = Some(adif_string(&adf)?);
            } else if adf.adif_name_canon == ADIF_HEADER_CREATED_TIMESTAMP {
                adif.adif_created_timestamp = Some(adif_string(&adf)?);
            }
        }
    }

    Ok(adif)
}

fn adif_string(adf: &AdiDataSpecifier) ->
    Result<String, AdifParseError>
{
    //
    // XXX We need to check the type specified with data specifier, but this
    // doesn't seem like the right way to do it.  Is it case-sensitive?  Are
    // there other string types?
    //
    if let Some(ref typestr) = adf.adif_type {
        if typestr != "S" {
            return Err(AdifParseError::ADIF_EBADINPUT(format!(
                "field \"{}\": expected string value, but found type \"{}\"",
                adf.adif_name, typestr)))
        }
    }

    // TODO is there a better pattern for the error handling pattern?
    // TODO extra copy
    match String::from_utf8(adf.adif_bytes.clone()) {
        Ok(s) => Ok(s),
        // TODO is there more useful information in this error?
        Err(_) => Err(AdifParseError::ADIF_EBADINPUT(format!(
                "field \"{}\": value contained invalid bytes for UTF-8 string",
                adf.adif_name)))
    }
}
