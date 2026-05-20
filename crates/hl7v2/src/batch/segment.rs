use super::BatchError;
use crate::model::{Atom, Comp, Field, Rep, Segment};

pub(crate) fn parse_segment(line: &str) -> Result<Segment, BatchError> {
    if line.len() < 3 {
        return Err(BatchError::InvalidStructure(format!(
            "Segment too short: {line}"
        )));
    }

    let Some(id_bytes) = line.as_bytes().get(0..3) else {
        return Err(BatchError::InvalidStructure(format!(
            "Segment too short: {line}"
        )));
    };
    let Ok(id) = <[u8; 3]>::try_from(id_bytes) else {
        return Err(BatchError::InvalidStructure(format!(
            "Segment too short: {line}"
        )));
    };

    let field_sep = line.chars().nth(3).unwrap_or('|');
    let field_strs: Vec<&str> = fields_after_separator(line).split(field_sep).collect();
    let fields: Vec<Field> = field_strs
        .iter()
        .map(|s| Field {
            reps: vec![Rep {
                comps: vec![Comp {
                    subs: vec![Atom::Text((*s).to_string())],
                }],
            }],
        })
        .collect();

    Ok(Segment { id, fields })
}

pub(crate) fn fields_after_separator(line: &str) -> &str {
    line.get(4..).unwrap_or_default()
}

pub(crate) fn segment_prefix(line: &str) -> &str {
    line.get(..3).unwrap_or(line)
}
