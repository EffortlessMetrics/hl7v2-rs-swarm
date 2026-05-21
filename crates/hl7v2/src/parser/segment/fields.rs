use crate::model::{Delims, Error, Field, Rep};

use super::components;

pub(super) fn parse_fields(fields_str: &str, delims: &Delims) -> Result<Vec<Field>, Error> {
    if fields_str.is_empty() {
        return Ok(vec![]);
    }

    let field_count = fields_str.matches(delims.field).count() + 1;
    let mut fields = Vec::with_capacity(field_count);

    for (i, field_str) in fields_str.split(delims.field).enumerate() {
        let field = parse_field(field_str, delims).map_err(|e| Error::ParseError {
            segment_id: "UNKNOWN".to_string(),
            field_index: i,
            source: Box::new(e),
        })?;
        fields.push(field);
    }

    Ok(fields)
}

fn parse_field(field_str: &str, delims: &Delims) -> Result<Field, Error> {
    if field_str.contains('\n') || field_str.contains('\r') {
        return Err(Error::InvalidFieldFormat {
            details: "Field contains invalid line break characters".to_string(),
        });
    }

    let rep_count = field_str.matches(delims.rep).count() + 1;
    let mut reps = Vec::with_capacity(rep_count);

    for (i, rep_str) in field_str.split(delims.rep).enumerate() {
        let rep = parse_rep(rep_str, delims).map_err(|e| match e {
            Error::InvalidRepFormat { .. } => e,
            _ => Error::InvalidRepFormat {
                details: format!("Repetition {}: {}", i, e),
            },
        })?;
        reps.push(rep);
    }

    Ok(Field { reps })
}

fn parse_rep(rep_str: &str, delims: &Delims) -> Result<Rep, Error> {
    components::parse_rep(rep_str, delims)
}
