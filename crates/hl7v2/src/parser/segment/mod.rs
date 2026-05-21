//! Segment parsing orchestrator.

#![expect(
    clippy::arithmetic_side_effects,
    clippy::uninlined_format_args,
    reason = "segment parsing preserves existing delimiter behavior while parser responsibilities are split into SRP submodules"
)]

mod components;
mod fields;
mod identity;

use crate::model::{Delims, Error, Segment};

pub(super) fn parse_segment(line: &str, delims: &Delims) -> Result<Segment, Error> {
    let id = identity::parse_segment_id(line)?;
    let fields_str = identity::extract_fields_str(line, delims)?;

    let mut fields = fields::parse_fields(fields_str, delims).map_err(|e| Error::ParseError {
        segment_id: String::from_utf8_lossy(&id).to_string(),
        field_index: 0,
        source: Box::new(e),
    })?;

    if &id == b"MSH"
        && let Some(first_field) = fields.first_mut()
    {
        *first_field = identity::msh_encoding_field(delims);
    }

    Ok(Segment { id, fields })
}
