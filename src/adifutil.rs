//
// src/adifutil.rs: utility functions for processing ADIF files
//

//
// Given a byte sequence "bytes" and a String "s", returns true if the "bytes"
// and "s" represent the same ASCII string when compared case-insensitively.
//
// This appears trickier than it sounds for a few reasons:
//
// - To compare case-insensitively, we need to be looking at Strings and chars,
//   not bytes.  That means we'll need to convert the bytes in "bytes" into
//   characters.
// - We don't want to fail if any of the bytes in "bytes" are non-ASCII.  That
//   makes using String::from_utf8 a bit verbose, since we have to match on the
//   result.  Plus, to use String::from_utf8, we'd need to clone "bytes", which
//   is fine in many cases, but we don't want to do that if "bytes" is huge and
//   "s" is tiny. (In practice, callers use this function where "s" is very
//   short, and it would be fine to copy either string as long as we know the
//   copy will be small.)
//
// This probably seems oddly specific.  The primary use-case is to identify
// short ASCII strings (like "eor") within arbitrarily large byte streams that
// may contain non-ASCII characters.  That in turn seems strange -- blame ADI.
//
pub fn byteseq_equal_ci(bytes: &Vec<u8>, s: &str) -> bool
{
    //
    // Rather than bother checking the size, cloning "bytes", converting to a
    // string, and comparing the result case-insensitively, we'll just compare
    // byte-by-byte.
    //
    if bytes.len() != s.len() {
        return false;
    }

    let strbytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let cb = bytes[i] as char;
        let cs = strbytes[i] as char;
        if !cs.eq_ignore_ascii_case(&cb) {
            return false;
        }
        i += 1;
    }

    return true;
}

// TODO add tests
