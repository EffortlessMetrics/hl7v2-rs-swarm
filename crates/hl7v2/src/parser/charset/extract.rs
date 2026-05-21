use crate::model::{Atom, Segment};

const MSH_18_INDEX: usize = 16;

/// Extract character sets from MSH-18 field.
pub(crate) fn extract_charsets(segments: &[Segment]) -> Vec<String> {
    let Some(field_18) = msh_18_field(segments) else {
        return vec![];
    };

    field_18
        .reps
        .iter()
        .filter_map(rep_primary_component_text)
        .collect()
}

fn msh_18_field(segments: &[Segment]) -> Option<&crate::model::Field> {
    let msh_segment = segments.first()?;
    if msh_segment.id.as_slice() != b"MSH" {
        return None;
    }
    msh_segment.fields.get(MSH_18_INDEX)
}

fn rep_primary_component_text(rep: &crate::model::Rep) -> Option<String> {
    rep.comps
        .first()
        .and_then(|comp| comp.subs.first())
        .and_then(non_empty_text)
}

fn non_empty_text(atom: &Atom) -> Option<String> {
    match atom {
        Atom::Text(text) if !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}
