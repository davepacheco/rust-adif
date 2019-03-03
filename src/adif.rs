//
// src/adif.rs: logical ADIF parser
//
// The facilities in this file convert the physical representation of an ADI
// file into a more useful interface for consumers.
//

use adi::AdiFile;
use adi::AdiDataSpecifier;
use super::AdifParseError;
use std::collections::BTreeMap;
use std::fmt;

// Well-known header fields
const ADIF_HEADER_ADIF_VER : &'static str = "adif_ver";
const ADIF_HEADER_CREATED_TIMESTAMP : &'static str = "created_timestamp";
const ADIF_HEADER_PROGRAMID : &'static str = "programid";
const ADIF_HEADER_PROGRAMVERSION : &'static str = "programversion";
const ADIF_HEADER_USERDEF : &'static str = "userdef";

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

#[allow(non_camel_case_types)]
pub enum AdifDumpWhichRecords {
    ADR_NONE,
    ADR_ONE,
    ADR_ALL
}

impl fmt::Debug for AdifFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ADIF file:  {}\n", self.adif_label)?;
        write!(f, "Created at: {}\n",
            match &self.adif_created_timestamp {
                Some(p) => p,
                None => "unknown"
            })?;
        write!(f, "Created by: {} {}\n",
            match &self.adif_program_id {
                Some(p) => format!("program \"{}\"", p),
                None => String::from("unknown program")
            },
            match &self.adif_program_version {
                Some(v) => format!("version \"{}\"", v),
                None => String::from("unknown version")
            })?;
        write!(f, "Total records: {}\n", self.adif_records.len())
    }
}

pub fn adif_dump(adif: AdifFile, which: AdifDumpWhichRecords,
    filterspec : &Option<Vec<(String, String)>>,
    colspec : &Option<Vec<&String>>)
{
    print!("{:?}", adif);

    match which {
        AdifDumpWhichRecords::ADR_NONE => (),
        AdifDumpWhichRecords::ADR_ONE => {
            print!("Example record:\n");
            adif_dump_one(&adif.adif_records[0], &None, colspec);
        },
        AdifDumpWhichRecords::ADR_ALL => {
            for rec in &adif.adif_records {
                adif_dump_one(rec, filterspec, &colspec);
            }
        }
    }
}

fn adif_dump_one(rec : &AdifRecord, filterspec: &Option<Vec<(String, String)>>,
    colspec: &Option<Vec<&String>>)
{
    if let Some(filters) = filterspec {
        for filter in filters {
            let key = &filter.0;
            let filterval = &filter.1;
            let recordentry = rec.adir_field_values.get(key);
            match recordentry {
                None => {
                    if filterval.len() > 0 {
                        return;
                    }
                },
                Some(recordval) => {
                    if filterval != recordval {
                        return;
                    }
                }
            }
        }
    }

    match colspec {
        None => print!("{:?}\n\n", rec),
        Some(colnames) => {
            for colname in colnames {
                let val = rec.adir_field_values.get(*colname);
                print!("{}\t", match val {
                    None => "-",
                    Some(v) => v
                });
            }
        }
    }

    print!("\n");
}

pub struct AdifRecord {
    pub adir_field_values : BTreeMap<String, String> // XXX value type?
}

impl fmt::Debug for AdifRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RECORD:\n")?;

        for (key, value) in self.adir_field_values.iter() {
            write!(f, "    {:20}: {}\n", key, value)?;
        }

        Ok(())
    }
}

// TODO Would this be better off accepting an iterator?
pub fn adif_parse_adi(label: &str, adi: &AdiFile) ->
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

    let mut which = 1;
    for adr in &adi.adi_records {
        let mut record_values : BTreeMap<String, String> = BTreeMap::new();

        for adf in &adr.adir_fields {
            // TODO presumably this is not legal ADIF?
            if record_values.contains_key(&adf.adif_name_canon) {
                return Err(AdifParseError::ADIF_EBADINPUT(format!(
                    "record {}: duplicate value for field \"{}\"", which,
                    adf.adif_name_canon)));
            }

            let value = adif_string(&adf)?;
            record_values.insert(adf.adif_name_canon.clone(), value);
        }

        which += 1;
        adif.adif_records.push(AdifRecord {
            adir_field_values : record_values
        });
    }

    Ok(adif)
}

//
// Given a data specifier describing a string-valued field, return a new String
// containing the field's contents.  This returns an error if the field is not
// string-valued or the value cannot be processed as UTF-8.
//
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
