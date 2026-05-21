use crate::model::Delims;

pub fn needs_escaping(text: &str, delims: &Delims) -> bool {
    text.contains(
        &[
            delims.field,
            delims.comp,
            delims.rep,
            delims.esc,
            delims.sub,
        ][..],
    )
}

pub fn needs_unescaping(text: &str, delims: &Delims) -> bool {
    text.contains(delims.esc)
}
