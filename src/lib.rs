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
struct AdiFile {
    pub adi_header : Option<AdiHeader>,   // file header, if present
    pub adi_records : Vec<AdiRecord>    // list of records in the file
}

//
// AdiHeader: represents the header in an ADI file, if present.
//

#[allow(non_camel_case_types)]
enum AdiHeaderDataSpecifierType {
    HST_ADIF_VERSION,   /* standard adif version field */
    HST_USERDEF,        /* user-defined field */
    HST_APP             /* application-defined field */
}

struct AdiHeaderDataSpecifier {
    pub adihf_fieldtype : AdiHeaderDataSpecifierType,
    pub adihf_name : String,
    pub adihf_name_canon : String,
    pub adihf_length : u64,
    pub adihf_bytes : String, // XXX
    pub adihf_type : Option<String> // XXX should be a Type enum
}

struct AdiHeader {
    pub adih_content : String,                      // complete header content
    pub adih_fields : Vec<AdiHeaderDataSpecifier>   // header data specifiers
}

//
// AdiRecord: represents a record in an ADI file.
//

struct AdiRecord {
    pub adir_fields : Vec<AdiDataSpecifier>
}

struct AdiDataSpecifier {
    pub adif_name : String,
    pub adif_name_canon : String,
    pub adif_length : u64,
    pub adif_bytes : String,    // XXX
    pub adif_type : Option<String> // XXX should be a Type enum
}

fn adi_export(adf : AdiFile) -> String {
    String::from("not yet implemented")
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

    #[test]
    pub fn do_stuff() {
        let adf = make_file_basic();
        println!("{}", super::adi_export(adf));
        // XXX test something
    }
}
