use crate::model::{Delims, Error};

/// Unescape text according to HL7 v2 rules.
pub fn unescape_text(text: &str, delims: &Delims) -> Result<String, Error> {
    let first_idx = match text.find(delims.esc) {
        Some(idx) => idx,
        None => return Ok(text.to_string()),
    };

    let mut result = String::with_capacity(text.len());
    result.push_str(&text[..first_idx]);

    let mut chars = text[first_idx..].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != delims.esc {
            result.push(ch);
            continue;
        }

        let mut escape_seq = String::new();
        let mut found_end = false;

        for esc_ch in chars.by_ref() {
            if esc_ch == delims.esc {
                found_end = true;
                break;
            }
            escape_seq.push(esc_ch);
        }

        if !found_end {
            if is_encoding_chars_literal(text, delims) {
                return Ok(text.to_string());
            }
            result.push(delims.esc);
            result.push_str(&escape_seq);
            continue;
        }

        append_decoded_escape(&mut result, &escape_seq, delims);
    }

    Ok(result)
}

fn is_encoding_chars_literal(text: &str, delims: &Delims) -> bool {
    if text.chars().count() != 4 {
        return false;
    }
    let chars: Vec<char> = text.chars().collect();
    chars[0] == delims.comp && chars[1] == delims.rep && chars[2] == delims.esc && chars[3] == delims.sub
}

fn append_decoded_escape(output: &mut String, escape_seq: &str, delims: &Delims) {
    match escape_seq {
        "F" => output.push(delims.field),
        "S" => output.push(delims.comp),
        "R" => output.push(delims.rep),
        "E" => output.push(delims.esc),
        "T" => output.push(delims.sub),
        _ => {
            output.push(delims.esc);
            output.push_str(escape_seq);
            output.push(delims.esc);
        }
    }
}
