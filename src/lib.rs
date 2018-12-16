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
    pub adih_fields : Vec<AdiDataSpecifier>         // header data specifiers
}

const ADI_STR_EOH : &'static str = "eoh";
const ADIF_HEADER_ADIF_VER : &'static str = "adif_ver";
const ADIF_HEADER_CREATED_TIMESTAMP : &'static str = "created_timestamp";
const ADIF_HEADER_PROGRAMID : &'static str = "programid";
const ADIF_HEADER_PROGRAMVERSION : &'static str = "programversion";
const ADIF_HEADER_USERDEF : &'static str = "userdef";
const ADI_MAX_FIELDLEN : usize = 1024;

//
// AdiRecord: represents a record in an ADI file.
//

pub struct AdiRecord {
    pub adir_fields : Vec<AdiDataSpecifier>
}

#[derive(Debug)]
pub struct AdiDataSpecifier {
    pub adif_name : String,
    pub adif_name_canon : String,
    pub adif_length : usize,
    pub adif_bytes : String,    // XXX
    pub adif_type : Option<String> // XXX should be a Type enum
}


//
// ADI Export
//
// These functions are used to export an ADI file.  At this point, they're
// intended for debugging rather than actual standards-compliant export.
//

fn adi_dump(adf : AdiFile) -> String {
    let mut output = String::new();

    match adf.adi_header {
        None => output.push_str("no header"),
        Some(adh) => {
            output.push_str(&adh.adih_content);
            for field in &adh.adih_fields {
                output.push_str(&format!("{:?}\n", field));
            }
            output.push_str("<eoh>\n");
        },
    }

    for rec in &adf.adi_records {
        adi_dump_record(rec, &mut output);
    }

    output
}

fn adi_dump_record(rec : &AdiRecord, output: &mut String) {
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
#[derive(Debug)]
pub enum AdiParseError {
    ADI_IO(io::Error),
    ADI_BADINPUT(String),
    ADI_NOT_YET_IMPLEMENTED(String),
}

impl From<io::Error> for AdiParseError {
    fn from(error: io::Error) -> Self {
        AdiParseError::ADI_IO(error)
    }
}

//
// Note: this type derives Clone because it's convenient for
// adi_parse_peek_token() to return a copy of its token.  However, that's pretty
// inefficient for string tokens.  It'd be nice to resolve this somehow.
#[derive(PartialEq, Debug, Clone)]
#[allow(non_camel_case_types)]
enum AdiToken {
    ADI_TOK_TEXT(String),
    ADI_TOK_LAB,    /* '<' */
    ADI_TOK_COLON,  /* ':' */
    ADI_TOK_RAB,    /* '>' */
    ADI_TOK_EOF
}

fn adi_token_text(token: &AdiToken) -> String
{
    String::from(match *token {
        AdiToken::ADI_TOK_TEXT(_) => "string",
        AdiToken::ADI_TOK_LAB => "\"<\"",
        AdiToken::ADI_TOK_RAB => "\">\"",
        AdiToken::ADI_TOK_COLON => "\":\"",
        AdiToken::ADI_TOK_EOF => "end of input"
    })
}

pub fn adi_import_test(input : &str) {
    println!("\n\ntokenizing string:\n{}\n\n", input);

    let source = Cursor::new(input);
    let mut buffered = BufReader::new(source);
    let mut maxiters = 100;

    loop {
        if maxiters == 0 {
            panic!("bailing out after max tokens reached!");
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
            Err(AdiParseError::ADI_NOT_YET_IMPLEMENTED(msg)) => {
                println!("not yet implemented: {}", msg);
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
    Result<AdiToken, AdiParseError>
{

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

fn adi_parse_string(source: &str) -> Result<AdiFile, AdiParseError> {
    let mut source_reader = Cursor::new(source);
    adi_parse(&mut source_reader)
}

// TODO needs work
struct AdiParseState<'a> {
    aps_source : Box<BufRead + 'a>,
    aps_tokens : Vec<AdiToken>,
    aps_error : bool,
    aps_done : bool
}

fn adi_parse_advance_tokens(aps : &mut AdiParseState, howmany : u8) ->
    Result<(), AdiParseError>
{
    assert!(!aps.aps_error);
    while !aps.aps_done && (howmany as usize) > aps.aps_tokens.len() {
        let result = adi_import_read_token(&mut aps.aps_source);
        match result {
            Ok(t) => {
                if t == AdiToken::ADI_TOK_EOF {
                    aps.aps_done = true;
                }
                aps.aps_tokens.push(t);
            }
            Err(e) => {
                aps.aps_error = true;
                return Err(e)
            }
        }
    }

    Ok(())
}

fn adi_parse_consume_tokens(aps : &mut AdiParseState, howmany : u8)
{
    //
    // It's illegal to try to consume tokens that haven't been read yet.
    // In order to read them, we must have loaded them into "aps_tokens".
    //
    assert!(!aps.aps_error);
    assert!(howmany as usize <= aps.aps_tokens.len());

    // TODO there's probably a more efficient way to do this.
    let mut count = 0;
    while count < howmany {
        aps.aps_tokens.remove(0);
        count += 1;
    }
}

fn adi_parse_peek_token<'a>(aps : &'a mut AdiParseState, which : u8) ->
    Result<AdiToken, AdiParseError>
{
    adi_parse_advance_tokens(aps, which + 1)?;

    let which = which as usize;
    if which < aps.aps_tokens.len() {
        return Ok(aps.aps_tokens[which].clone());
    }

    //
    // At this point, we must be at end-of-file, and the last token ought to be
    // the end-of-file token.
    //
    assert!(aps.aps_done);
    assert!(aps.aps_tokens.len() > 0);
    assert_eq!(aps.aps_tokens[aps.aps_tokens.len() - 1], AdiToken::ADI_TOK_EOF);
    return Ok(aps.aps_tokens[aps.aps_tokens.len() - 1].clone());
}

fn adi_parse(source: &mut std::io::Read) -> Result<AdiFile, AdiParseError>
{
    let mut aps = AdiParseState {
        aps_source: Box::new(BufReader::new(source)),
        aps_tokens: Vec::new(),
        aps_error: false,
        aps_done: false
    };

    let header = match adi_parse_peek_token(&mut aps, 0)? {
        AdiToken::ADI_TOK_LAB => None,
        _ => {
            Some(adi_parse_header(&mut aps)?)
        }
    };

    Ok(AdiFile {
        adi_header: header,
        adi_records: vec![] // XXX
    })
}

fn adi_parse_header(aps: &mut AdiParseState) -> Result<AdiHeader, AdiParseError>
{
    let mut header_content = String::new();
    let mut header_fields : Vec<AdiDataSpecifier> = Vec::new();

    loop {
        match adi_parse_peek_token(aps, 0)? {
            AdiToken::ADI_TOK_TEXT(s) => {
                header_content.push_str(s.as_str());
                adi_parse_consume_tokens(aps, 1);
            },

            //
            // Although it seems crazy, the ADIF specification does not say
            // anything wrong with having these special characters loose in the
            // header (i.e., not following a "<").  We thus treat these as
            // plain text.
            // TODO record a warning?
            //
            AdiToken::ADI_TOK_COLON => {
                header_content.push_str(":");
                adi_parse_consume_tokens(aps, 1);
            },
            AdiToken::ADI_TOK_RAB => {
                header_content.push_str(">");
                adi_parse_consume_tokens(aps, 1);
            },

            AdiToken::ADI_TOK_LAB => {
                let next = adi_parse_peek_token(aps, 1)?;
                if let AdiToken::ADI_TOK_TEXT(s) = &next {
                    if s.to_lowercase() == ADI_STR_EOH {
                        let next2 = adi_parse_peek_token(aps, 2)?;
                        if next2 == AdiToken::ADI_TOK_RAB {
                            //
                            // We're done with the header.  Consume the '<',
                            // "eoh", and ">" tokens and move on.
                            //
                            adi_parse_consume_tokens(aps, 3);
                            break;
                        }
                    }
                }

                //
                // If we make it here, it's because we got something other than
                // "<eoh>".  Parse this as a data specifier.  Note that this
                // means we'd support a normal data specifier for a field called
                // "eoh", which is pretty dubious, but appears to be technically
                // allowed.
                //
                let spec = adi_parse_data_specifier(aps)?;
                header_fields.push(spec);
            },

            AdiToken::ADI_TOK_EOF => {
                return Err(AdiParseError::ADI_BADINPUT(
                    "unexpected end of input while reading header".to_string()));
            }
        }
    }

    Ok(AdiHeader {
        adih_content: header_content,
        adih_fields: header_fields
    })
}

//
// There are two valid token sequences here.  Below is the simple case:
//
//   <FIELDNAME:FIELDLEN>FIELDVALUE_...<
//   ^^        ^^       ^^         ^   ^
//   ||        ||       ||         |   | # TOKEN
//   ++--------++-------++---------+---+ 0 (LAB)
//    +--------++-------++---------+---+ 1 (STRING) FIELDNAME
//             ++-------++---------+---+ 2 (COLON)
//              +-------++---------+---+ 3 (STRING) FIELDLEN
//                      ++---------+---+ 4 (RAB)
//                       +---------+---+ 5 (STRING) FIELDVALUE
//                                 +---+ 6 (STRING) (discarded)
//                                     + 7 (LAB)
//
// ADI also allows an additional colon (COLON) and type specifier (STRING)
// directly after the field length.  We do not yet support this, so we only
// handle the sequence above.
//
// The caller is responsible for ensuring that the first token is a left angle
// bracket before invoking this function.
//
fn adi_parse_data_specifier(aps : &mut AdiParseState) ->
    Result<AdiDataSpecifier, AdiParseError>
{
    assert_eq!(adi_parse_peek_token(aps, 0).unwrap(), AdiToken::ADI_TOK_LAB);

    let t_fieldname   = adi_parse_peek_token(aps, 1)?;
    let t_colon       = adi_parse_peek_token(aps, 2)?;
    let t_fieldlength = adi_parse_peek_token(aps, 3)?;
    let t_rab         = adi_parse_peek_token(aps, 4)?;

    let fieldname = match t_fieldname {
        AdiToken::ADI_TOK_TEXT(s) => s,
        _ => {
            return Err(AdiParseError::ADI_BADINPUT(format!(
                "parsing data specifier: expected string for field name, but \
                found {}", adi_token_text(&t_fieldname))));
        }
    };

    match t_colon {
        AdiToken::ADI_TOK_COLON => (),
        _ => {
            return Err(AdiParseError::ADI_BADINPUT(format!(
                "parsing data specifier: expected {}, but found {}",
                adi_token_text(&AdiToken::ADI_TOK_COLON),
                adi_token_text(&t_colon))));
        }
    };

    let fieldlength_str = match t_fieldlength {
        AdiToken::ADI_TOK_TEXT(s) => s,
        _ => {
            return Err(AdiParseError::ADI_BADINPUT(format!(
                "parsing data specifier length: expected field length, \
                but found {}", adi_token_text(&t_fieldlength))));
        }
    };

    let fieldlength_result = fieldlength_str.parse::<usize>();
    let fieldlength = match fieldlength_result {
        Ok(n) if n <= ADI_MAX_FIELDLEN => n,
        Ok(_) => {
            //
            // This limit is not intrinsic to our approach, but it's intended to
            // ensure that we fail gracefully if given something that would
            // otherwise attempt to use lots of memory.
            //
            return Err(AdiParseError::ADI_BADINPUT(format!(
                "parsing data specifier: max supported size is {} bytes",
                ADI_MAX_FIELDLEN)));
        }
        Err(s) => {
            return Err(AdiParseError::ADI_BADINPUT(format!(
                "parsing data specifier length: {}", s)));
        }
    };

    match t_rab {
        AdiToken::ADI_TOK_RAB => (),
        AdiToken::ADI_TOK_COLON => {
            // TODO
            return Err(AdiParseError::ADI_NOT_YET_IMPLEMENTED(String::from(
                "parsing data specifier: typed values are not supported")));
        },
        _ => {
            return Err(AdiParseError::ADI_BADINPUT(format!(
                "parsing data specifier: expected {}, but found {}",
                adi_token_text(&AdiToken::ADI_TOK_RAB),
                adi_token_text(&t_rab))));
        }
    };

    adi_parse_consume_tokens(aps, 5);
    let mut fieldvalue = String::new();
    while fieldlength > fieldvalue.len() {
        let t_value = adi_parse_peek_token(aps, 0)?;
        adi_parse_consume_tokens(aps, 1);
        match t_value {
            AdiToken::ADI_TOK_COLON => {
                fieldvalue.push_str("<");
            }
            AdiToken::ADI_TOK_RAB => {
                fieldvalue.push_str(">");
            }
            AdiToken::ADI_TOK_LAB => {
                fieldvalue.push_str("<");
            }
            AdiToken::ADI_TOK_TEXT(s) => {
                fieldvalue.push_str(&s);
            }
            AdiToken::ADI_TOK_EOF => {
                return Err(AdiParseError::ADI_BADINPUT(format!(
                    "parsing data specifier: unexpected {} in value",
                    adi_token_text(&AdiToken::ADI_TOK_EOF))));
            }
        }
    }

    //
    // At this point, we've read enough bytes to account for the value.  Discard
    // anything extra that we've accumulated.
    // TODO this only works for ASCII.
    //
    fieldvalue.truncate(fieldlength);

    Ok(AdiDataSpecifier {
        adif_name_canon: fieldname.to_lowercase(),
        adif_name: fieldname.to_string(), // TODO extra copy?
        adif_length: fieldlength,
        adif_bytes: fieldvalue,
        adif_type: None
    })
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
            adih_fields: vec![ super::AdiDataSpecifier {
                adif_name: String::from("adif_VERSion"),
                adif_name_canon: String::from("adif_version"),
                adif_length: 3,
                adif_bytes: String::from("1.0"),
                adif_type: None
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
        println!("{}", super::adi_dump(adf));
        let adf = make_file_header();
        println!("{}", super::adi_dump(adf));
        let adf = make_file_complex();
        println!("{}", super::adi_dump(adf));
        super::adi_import_test(r#"
            header stuff here<eoh>
            <call:6>KK6ZBI
            <bupkis:3>123
            <eor>
        "#);

        println!("\n\nparse tests\n");
        parse_test_string(r"foobar");
        parse_test_string(r"<foobar>");
        parse_test_string(r"<eoh>"); // should be disallowed later
        parse_test_string(r"foobar<eoh>");
        parse_test_string(r"foobar<eoh:3>789");
        parse_test_string(r"foobar<foo:3>123<eoh>");
        parse_test_string(r"preamble<foo:3>12345<bar:7>123456789<eoh>");

        // XXX test something
    }

    fn parse_test_string(s : &str) {
        println!("test input:\n{}\n", s);
        test_print(super::adi_parse_string(s));
    }

    fn test_print(r : Result<super::AdiFile, super::AdiParseError>) {
        match r {
            Err(e) => {
                println!("error:\n{:?}", e);
            },
            Ok(adf) => {
                println!("success:\n{}", super::adi_dump(adf));
            }
        }
    }
}
