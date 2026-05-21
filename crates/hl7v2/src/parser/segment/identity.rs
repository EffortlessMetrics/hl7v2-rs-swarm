use crate::model::{Atom, Comp, Delims, Error, Field, Rep};

pub(super) fn parse_segment_id(line: &str) -> Result<[u8; 3], Error> {
    if line.len() < 3 {
        std::hint::cold_path();
        return Err(Error::InvalidSegmentId);
    }

    let Some(id_bytes) = line.as_bytes().get(0..3) else {
        std::hint::cold_path();
        return Err(Error::InvalidSegmentId);
    };

    let mut id = [0u8; 3];
    id.copy_from_slice(id_bytes);

    for &byte in &id {
        if !(byte.is_ascii_uppercase() || byte.is_ascii_digit()) {
            std::hint::cold_path();
            return Err(Error::InvalidSegmentId);
        }
    }

    Ok(id)
}

pub(super) fn extract_fields_str<'a>(line: &'a str, delims: &Delims) -> Result<&'a str, Error> {
    let mut field_sep_buf = [0; 4];
    let field_sep = delims.field.encode_utf8(&mut field_sep_buf);

    if line.len() == 3 {
        return Ok("");
    }

    if field_sep.len() == 1 && line.as_bytes().get(3) == field_sep.as_bytes().first() {
        let Some(fields_str) = line.get(4..) else {
            std::hint::cold_path();
            return Err(Error::InvalidFieldFormat {
                details: "Segment field separator must end at a UTF-8 boundary".to_string(),
            });
        };
        return Ok(fields_str);
    }

    std::hint::cold_path();
    Err(Error::InvalidFieldFormat {
        details: "Segment fields must start with the configured field separator".to_string(),
    })
}

pub(super) fn msh_encoding_field(delims: &Delims) -> Field {
    let encoding_chars = String::from_iter([delims.comp, delims.rep, delims.esc, delims.sub]);

    Field {
        reps: vec![Rep {
            comps: vec![Comp {
                subs: vec![Atom::Text(encoding_chars)],
            }],
        }],
    }
}
