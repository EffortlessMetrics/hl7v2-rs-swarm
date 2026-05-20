use crate::model::{Atom, Field, Rep};
use std::collections::BTreeMap;

use super::CorpusValueShapeStats;

pub(super) fn field_is_present(field: &Field) -> bool {
    field.reps.iter().any(|rep| {
        rep.comps.iter().any(|comp| {
            comp.subs.iter().any(|atom| match atom {
                Atom::Text(text) => !text.is_empty(),
                Atom::Null => true,
            })
        })
    })
}

pub(super) fn increment_count(counts: &mut BTreeMap<String, usize>, value: String) {
    let count = counts.entry(value).or_insert(0);
    *count = count.saturating_add(1);
}

#[derive(Clone, Copy)]
enum ValueShape {
    Coded,
    Timestamp,
    Numeric,
    Null,
    Text,
}

pub(super) fn record_value_shape(
    value_shapes: &mut BTreeMap<String, CorpusValueShapeStats>,
    path: &str,
    field: &Field,
) {
    let stats = value_shapes
        .entry(path.to_string())
        .or_insert_with(|| empty_value_shape_stats(path));
    for shape in field_value_shapes(field) {
        match shape {
            ValueShape::Coded => stats.coded_count = stats.coded_count.saturating_add(1),
            ValueShape::Timestamp => {
                stats.timestamp_count = stats.timestamp_count.saturating_add(1);
            }
            ValueShape::Numeric => stats.numeric_count = stats.numeric_count.saturating_add(1),
            ValueShape::Null => stats.null_count = stats.null_count.saturating_add(1),
            ValueShape::Text => stats.text_count = stats.text_count.saturating_add(1),
        }
    }
}

fn empty_value_shape_stats(path: &str) -> CorpusValueShapeStats {
    CorpusValueShapeStats {
        path: path.to_string(),
        coded_count: 0,
        timestamp_count: 0,
        numeric_count: 0,
        null_count: 0,
        text_count: 0,
    }
}

fn field_value_shapes(field: &Field) -> Vec<ValueShape> {
    field.reps.iter().filter_map(repetition_value_shape).collect()
}

fn repetition_value_shape(rep: &Rep) -> Option<ValueShape> {
    if rep
        .comps
        .iter()
        .flat_map(|component| component.subs.iter())
        .any(|atom| matches!(atom, Atom::Null))
    {
        return Some(ValueShape::Null);
    }

    if rep.comps.len() > 1 {
        return Some(ValueShape::Coded);
    }

    let text = rep.first_text()?;
    if text.is_empty() {
        return None;
    }

    if is_hl7_timestamp_shape(text) {
        Some(ValueShape::Timestamp)
    } else if text.parse::<f64>().is_ok() {
        Some(ValueShape::Numeric)
    } else {
        Some(ValueShape::Text)
    }
}

fn is_hl7_timestamp_shape(text: &str) -> bool {
    matches!(text.len(), 8 | 12 | 14) && text.chars().all(|character| character.is_ascii_digit())
}
