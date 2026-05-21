use crate::model::Delims;

/// Escape text according to HL7 v2 rules.
pub fn escape_text(text: &str, delims: &Delims) -> String {
    let delims_arr = [
        delims.field,
        delims.comp,
        delims.rep,
        delims.esc,
        delims.sub,
    ];

    let first_idx = match text.find(&delims_arr[..]) {
        Some(idx) => idx,
        None => return text.to_string(),
    };

    let mut result = String::with_capacity(text.len() + 10);
    result.push_str(&text[..first_idx]);

    for ch in text[first_idx..].chars() {
        match ch {
            c if c == delims.field => push_escape(&mut result, delims.esc, 'F'),
            c if c == delims.comp => push_escape(&mut result, delims.esc, 'S'),
            c if c == delims.rep => push_escape(&mut result, delims.esc, 'R'),
            c if c == delims.esc => push_escape(&mut result, delims.esc, 'E'),
            c if c == delims.sub => push_escape(&mut result, delims.esc, 'T'),
            _ => result.push(ch),
        }
    }

    result
}

fn push_escape(result: &mut String, esc: char, code: char) {
    result.push(esc);
    result.push(code);
    result.push(esc);
}
