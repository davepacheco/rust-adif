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

// Types of ADIF files.  See above.
#[allow(non_camel_case_types)]
enum AdifFileType {
    ADIF_FT_ADI,    /* ADI (forward-compatible from v1) */
    ADIF_FT_ADX,    /* ADX (not supported) */
}

//
// ADI format: physical elements
//
// The following structures describe the low-level elements of an ADI file
// (i.e., header, data specifier, record, etc.).  The semantics of these
// elements will be interpreted at a higher level by the ADIF parser.  At this
// level, we're not interpreting much -- we're doing our best to describe the
// low-level ADI elements.
//

//
// AdiFile: represents a complete ADI file.  This structure is not compatible
// with a streaming parser, but we're not looking to build one here.
//
pub struct AdiFile {
    pub adi_header : Option<AdiHeader>,   // file header, if present
    pub adi_records : Vec<AdiRecord>    // list of records in the file
}

//
// AdiHeader: represents the header in an ADI file, if present.
//

pub struct AdiHeader {
    pub adih_content : String,                      // complete header content
    pub adih_fields : Vec<AdiHeaderDataSpecifier>   // header data specifiers
}

#[allow(non_camel_case_types)]
pub enum AdiHeaderDataSpecifierType {
    HST_ADIF_VERSION,   /* standard adif version field */
    HST_USERDEF,        /* user-defined field */
    HST_APP             /* application-defined field */
}

pub struct AdiHeaderDataSpecifier {
    pub adihf_fieldtype : AdiHeaderDataSpecifierType,
    pub adihf_name : String,
    pub adihf_name_canon : String,
    pub adihf_length : u64,
    pub adihf_bytes : String, // XXX
    pub adihf_type : Option<String> // XXX should be a Type enum
}

//
// AdiRecord: represents a record in an ADI file.
//

pub struct AdiRecord {
    pub adir_fields : Vec<AdiDataSpecifier>
}

pub struct AdiDataSpecifier {
    pub adif_name : String,
    pub adif_name_canon : String,
    pub adif_length : u64,
    pub adif_bytes : String,    // XXX
    pub adif_type : Option<String> // XXX should be a Type enum
}


//
// ADI Export
//
// These functions are used to export an ADI file.  At this point, they're
// intended for debugging rather than actual standards-compliant export.
//

// TODO this is an unpolished API for playing around.
fn adi_export(adf : AdiFile) -> String {
    let mut output = String::new();

    match adf.adi_header {
        None => output.push_str("no header"),
        Some(adh) => {
            output.push_str(&adh.adih_content);
            output.push_str("<eoh>\n");
        },
    }

    for rec in &adf.adi_records {
        adi_export_record(rec, &mut output);
    }

    output
}

fn adi_export_record(rec : &AdiRecord, output: &mut String) {
    for field in &rec.adir_fields {
        output.push_str(format!("    <{}:{}", field.adif_name_canon.as_str(),
            field.adif_length.to_string().as_str()).as_str());
        if let Some(t) = &field.adif_type {
            output.push_str(":");
            output.push_str(t.as_str());
        }
        output.push_str(">");
        output.push_str(field.adif_bytes.to_string().as_str());
        output.push_str("\n");
    }
    output.push_str("<eor>\n");
}


//
// ADI Import
//
// These structures and functions are used to import an ADI file.
//

use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Cursor;

#[allow(non_camel_case_types)]
pub enum AdiParseError {
    ADI_IO(io::Error),
    ADI_BADINPUT(String),
    ADI_NOT_YET_IMPLEMENTED
}

impl From<io::Error> for AdiParseError {
    fn from(error: io::Error) -> Self {
        AdiParseError::ADI_IO(error)
    }
}

#[derive(PartialEq)]
#[allow(non_camel_case_types)]
enum AdiToken {
    ADI_TOK_TEXT(String),
    ADI_TOK_LAB,    /* '<' */
    ADI_TOK_COLON,  /* ':' */
    ADI_TOK_RAB,    /* '>' */
    ADI_TOK_EOF
}

pub fn adi_import_test(input : &str) {
    println!("\n\ntokenzing string:\n{}\n\n", input);

    let source = Cursor::new(input);
    let mut buffered = BufReader::new(source);
    let mut maxiters = 100;

    loop {
        if maxiters == 0 {
            panic!("bailing out after max tokens reached!");
            return;
        }
        maxiters -= 1;

        let rtoken = adi_import_read_token(&mut buffered);
        match rtoken {
            Err(AdiParseError::ADI_IO(ioe)) => {
                println!("unexpected I/O error: {}", ioe);
                return;
            },
            Err(AdiParseError::ADI_BADINPUT(msg)) => {
                println!("bad input: {}", msg);
                return;
            },
            Err(AdiParseError::ADI_NOT_YET_IMPLEMENTED) => {
                println!("not yet implemented");
                return;
            },

            Ok(AdiToken::ADI_TOK_LAB) => {
                println!("token: '<'");
            },
            Ok(AdiToken::ADI_TOK_RAB) => {
                println!("token: '>'");
            },
            Ok(AdiToken::ADI_TOK_COLON) => {
                println!("token: ':'");
            },
            Ok(AdiToken::ADI_TOK_TEXT(t)) => {
                 println!("token: raw text: {}", t);
            },
            Ok(AdiToken::ADI_TOK_EOF) => {
                println!("token: EOF");
                return;
            }
        }
    }
}

fn adi_import_read_token(source : &mut BufRead) ->
    Result<AdiToken, AdiParseError> {

    let c = {
        let buf = source.fill_buf()?;
        if buf.len() == 0 {
            return Ok(AdiToken::ADI_TOK_EOF);
        }

        buf[0] as char
    };

    if c == '<' {
        source.consume(1);
        return Ok(AdiToken::ADI_TOK_LAB);
    }

    if c == ':' {
        source.consume(1);
        return Ok(AdiToken::ADI_TOK_COLON);
    }

    if c == '>' {
        source.consume(1);
        return Ok(AdiToken::ADI_TOK_RAB);
    }

    let (text, length) = {
        let buf = source.fill_buf()?;
        let mut i = 0;
        loop {
            let c = buf[i] as char;

            //
            // TODO This validation is going to eventually have to move to a
            // higher level of parsing, because technically arbitrary byte
            // sequences can appear here.
            // There also seems to be a bug in the spec, in that the header is
            // defined to be a String, yet the examples contain newlines.  This
            // appears intended to include MultilineString.
            // Even so, the following validation isn't quite right, because
            // MultiLine strings are only allowed to contain CR/LF
            // consecutively, not a naked CR or LF or the reverse order, but
            // we're not validating that here.
            //
            if !c.is_ascii() ||
               (c.is_ascii_control() && c != '\r' && c != '\n') {
                // TODO add byte offset
                return Err(AdiParseError::ADI_BADINPUT(format!(
                    "expected ASCII character, but found byte 0x{:x}",
                    buf[i])));
            }

            if c == '<' || c == ':' || c == '>' {
                break;
            }

            i += 1;
            if i == buf.len() {
                break;
            }
        }

        //
        // It's impossible for String::from_utf8() to fail here, since we've
        // already validated that every character is ASCII.
        //
        (String::from_utf8(buf[0..i].to_vec()).unwrap(), i)
    };

    source.consume(length);
    Ok(AdiToken::ADI_TOK_TEXT(text))
}


#[cfg(test)]
mod test {
    fn make_file_basic() -> super::AdiFile {
        let header = None;
        let records = vec![];
        return super::AdiFile {
            adi_header: header,
            adi_records: records
        }
    }

    fn make_file_header() -> super::AdiFile {
        let headerstr = String::from("This is a test file!\n");
        let header = super::AdiHeader {
            adih_content: headerstr,
            adih_fields: vec![]
        };
        let records = vec![];
        return super::AdiFile {
            adi_header: Some(header),
            adi_records: records
        }
    }

    fn make_file_complex() -> super::AdiFile {
        let headerstr = String::from(
            r#"This is a string.<adif_VERSion:3>1.0\nMore content"#);
        let header = super::AdiHeader {
            adih_content: headerstr,
            adih_fields: vec![ super::AdiHeaderDataSpecifier {
                adihf_fieldtype: super::AdiHeaderDataSpecifierType::HST_ADIF_VERSION,
                adihf_name: String::from("adif_VERSion"),
                adihf_name_canon: String::from("adif_version"),
                adihf_length: 3,
                adihf_bytes: String::from("1.0"),
                adihf_type: None
            } ]
        };
        let records = vec![
            super::AdiRecord {
                adir_fields: vec![
                    super::AdiDataSpecifier {
                        adif_name: String::from("call"),
                        adif_name_canon: String::from("call"),
                        adif_length: 6,
                        adif_bytes: String::from("KK6ZBI"),
                        adif_type: None
                    },

                    super::AdiDataSpecifier {
                        adif_name: String::from("QSO_date"),
                        adif_name_canon: String::from("qso_date"),
                        adif_length: 8,
                        adif_bytes: String::from("20181129"),
                        adif_type: None
                    }
                ]
            },
            super::AdiRecord {
                adir_fields: vec![
                    super::AdiDataSpecifier {
                        adif_name: String::from("call"),
                        adif_name_canon: String::from("call"),
                        adif_length: 6,
                        adif_bytes: String::from("KB1HCN"),
                        adif_type: Some(String::from("S"))
                    },

                    super::AdiDataSpecifier {
                        adif_name: String::from("QSO_date"),
                        adif_name_canon: String::from("qso_date"),
                        adif_length: 8,
                        adif_bytes: String::from("20181130"),
                        adif_type: None
                    }
                ]
            }
        ];
        return super::AdiFile {
            adi_header: Some(header),
            adi_records: records
        }
    }

    #[test]
    pub fn do_stuff() {
        let adf = make_file_basic();
        println!("{}", super::adi_export(adf));
        let adf = make_file_header();
        println!("{}", super::adi_export(adf));
        let adf = make_file_complex();
        println!("{}", super::adi_export(adf));
        super::adi_import_test(r#"
            header stuff here<eoh>
            <call:6>KK6ZBI
            <bupkis:3>123
            <eor>
        "#);
        // XXX test something
    }
}
