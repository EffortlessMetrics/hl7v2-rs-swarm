//! HL7 v2 test corpus generation and management utilities.
//!
//! This module provides functionality for managing test corpora of HL7 v2 messages.
//! It includes:
//!
//! - Manifest handling for reproducible test data
//! - Golden hash verification for regression testing
//! - Train/validation/test split management
//! - SHA-256 hash computation utilities
//!
//! # Manifest Management
//!
//! The [`CorpusManifest`] type tracks all metadata needed for reproducible
//! corpus generation:
//!
//! - Templates and their hashes
//! - Generation seed
//! - Message metadata
//! - Train/validation/test splits
//!
//! # Example
//!
//! ```
//! use hl7v2::synthetic::corpus::{CorpusManifest, compute_sha256};
//!
//! let mut manifest = CorpusManifest::new(42);
//! manifest.add_template("test.yaml", "template content");
//! manifest.add_message("msg001.hl7", "MSH|^~\\&|...", "ADT^A01", 0);
//!
//! let json = manifest.to_json().unwrap();
//! let parsed = CorpusManifest::from_json(&json).unwrap();
//! assert_eq!(parsed.seed, 42);
//! ```

use crate::model::{Atom, Field, Message};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

mod diff;
mod hash;
mod manifest;
mod source;

pub use diff::{diff_corpus_fingerprints, diff_corpus_paths};
pub use hash::{compute_message_hash, compute_sha256};
pub use manifest::{
    CorpusConfig, CorpusManifest, CorpusSplits, MessageInfo, ProfileInfo, TemplateInfo,
};
use source::{collect_corpus_files, parse_corpus_message_bytes, relative_corpus_path};

/// Error type for corpus operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum CorpusError {
    /// Error during serialization/deserialization
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error during file I/O
    #[error("IO error: {0}")]
    IoError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Invalid split ratios
    #[error("Invalid split ratios: must sum to 1.0")]
    InvalidSplitRatios,
}

/// Summarize a file or directory of HL7 v2 messages.
///
/// Directories are scanned recursively. Each regular file is read and parsed as
/// plain HL7 unless it is MLLP framed. Files that fail to parse are recorded in
/// the returned summary rather than failing the whole operation.
///
/// # Errors
///
/// Returns [`CorpusError::InvalidConfig`] if the path is neither a regular file
/// nor a directory. Returns [`CorpusError::IoError`] if directory traversal or
/// file reading fails.
pub fn summarize_corpus_path(path: impl AsRef<Path>) -> Result<CorpusSummary, CorpusError> {
    let root = path.as_ref();
    let mut files = Vec::new();
    collect_corpus_files(root, &mut files)?;
    files.sort();

    let mut message_type_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut segment_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut field_message_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut field_occurrence_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut parse_errors = Vec::new();
    let mut total_bytes = 0usize;
    let mut message_count = 0usize;

    for file in &files {
        let relative_path = relative_corpus_path(root, file);
        let bytes =
            fs::read(file).map_err(|e| CorpusError::IoError(format!("{relative_path}: {e}")))?;
        total_bytes = total_bytes.saturating_add(bytes.len());

        match parse_corpus_message_bytes(&bytes) {
            Ok(message) => {
                message_count = message_count.saturating_add(1);
                increment_count(&mut message_type_counts, extract_message_type(&message));
                record_message_shape(
                    &message,
                    &mut segment_counts,
                    &mut field_message_counts,
                    &mut field_occurrence_counts,
                );
            }
            Err(error) => parse_errors.push(CorpusParseFailure {
                path: relative_path,
                error: error.to_string(),
            }),
        }
    }

    let mut field_presence: Vec<CorpusFieldPresence> = field_occurrence_counts
        .into_iter()
        .map(|(path, occurrence_count)| CorpusFieldPresence {
            message_count: field_message_counts.get(&path).copied().unwrap_or_default(),
            path,
            occurrence_count,
        })
        .collect();
    field_presence.sort_by(|left, right| compare_field_paths(&left.path, &right.path));

    Ok(CorpusSummary {
        root: root.to_string_lossy().to_string(),
        file_count: files.len(),
        message_count,
        parse_error_count: parse_errors.len(),
        total_bytes,
        message_types: counts_to_vec(message_type_counts),
        segments: counts_to_vec(segment_counts),
        field_presence,
        parse_errors,
    })
}

/// Summarize an in-memory corpus of HL7 v2 messages.
///
/// Each message is parsed as plain HL7 unless it is MLLP framed. Messages that
/// fail to parse are recorded in the returned summary rather than failing the
/// whole operation.
#[must_use]
pub fn summarize_corpus_messages(
    root: impl Into<String>,
    messages: &[CorpusMessageRef<'_>],
) -> CorpusSummary {
    let mut message_type_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut segment_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut field_message_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut field_occurrence_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut parse_errors = Vec::new();
    let mut total_bytes = 0usize;
    let mut message_count = 0usize;

    for message_ref in messages {
        total_bytes = total_bytes.saturating_add(message_ref.bytes.len());

        match parse_corpus_message_bytes(message_ref.bytes) {
            Ok(message) => {
                message_count = message_count.saturating_add(1);
                increment_count(&mut message_type_counts, extract_message_type(&message));
                record_message_shape(
                    &message,
                    &mut segment_counts,
                    &mut field_message_counts,
                    &mut field_occurrence_counts,
                );
            }
            Err(error) => parse_errors.push(CorpusParseFailure {
                path: message_ref.id.to_string(),
                error: error.to_string(),
            }),
        }
    }

    let mut field_presence: Vec<CorpusFieldPresence> = field_occurrence_counts
        .into_iter()
        .map(|(path, occurrence_count)| CorpusFieldPresence {
            message_count: field_message_counts.get(&path).copied().unwrap_or_default(),
            path,
            occurrence_count,
        })
        .collect();
    field_presence.sort_by(|left, right| compare_field_paths(&left.path, &right.path));

    CorpusSummary {
        root: root.into(),
        file_count: messages.len(),
        message_count,
        parse_error_count: parse_errors.len(),
        total_bytes,
        message_types: counts_to_vec(message_type_counts),
        segments: counts_to_vec(segment_counts),
        field_presence,
        parse_errors,
    }
}

/// Fingerprint a file or directory of HL7 v2 messages.
///
/// The fingerprint is a compact deterministic feed signature derived from the
/// parsed messages in the corpus. Parse failures are counted but skipped for
/// shape-level dimensions.
///
/// # Errors
///
/// Returns [`CorpusError::InvalidConfig`] if the path is neither a regular file
/// nor a directory. Returns [`CorpusError::IoError`] if traversal or file
/// reading fails.
pub fn fingerprint_corpus_path(path: impl AsRef<Path>) -> Result<CorpusFingerprint, CorpusError> {
    let root = path.as_ref();
    let summary = summarize_corpus_path(root)?;
    let (field_cardinality, value_shape_stats) =
        collect_fingerprint_details(root, summary.message_count)?;

    Ok(CorpusFingerprint {
        fingerprint_version: "1".to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        root: summary.root,
        profile: None,
        file_count: summary.file_count,
        message_count: summary.message_count,
        parse_error_count: summary.parse_error_count,
        message_type_counts: summary.message_types,
        segment_counts: summary.segments,
        field_presence: summary.field_presence,
        field_cardinality,
        value_shape_stats,
        validation_issue_code_counts: Vec::new(),
    })
}

/// Fingerprint an in-memory corpus of HL7 v2 messages.
///
/// The fingerprint is a compact deterministic feed signature derived from the
/// parsed messages in the corpus. Parse failures are counted but skipped for
/// shape-level dimensions.
#[must_use]
pub fn fingerprint_corpus_messages(
    root: impl Into<String>,
    messages: &[CorpusMessageRef<'_>],
) -> CorpusFingerprint {
    let summary = summarize_corpus_messages(root, messages);
    let (field_cardinality, value_shape_stats) =
        collect_fingerprint_details_from_messages(messages, summary.message_count);

    CorpusFingerprint {
        fingerprint_version: "1".to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        root: summary.root,
        profile: None,
        file_count: summary.file_count,
        message_count: summary.message_count,
        parse_error_count: summary.parse_error_count,
        message_type_counts: summary.message_types,
        segment_counts: summary.segments,
        field_presence: summary.field_presence,
        field_cardinality,
        value_shape_stats,
        validation_issue_code_counts: Vec::new(),
    }
}

fn collect_fingerprint_details(
    root: &Path,
    message_count: usize,
) -> Result<(Vec<CorpusFieldCardinality>, Vec<CorpusValueShapeStats>), CorpusError> {
    let mut files = Vec::new();
    collect_corpus_files(root, &mut files)?;
    files.sort();

    let mut cardinality: BTreeMap<String, FieldCardinalityAccumulator> = BTreeMap::new();
    let mut value_shapes: BTreeMap<String, CorpusValueShapeStats> = BTreeMap::new();

    for file in &files {
        let relative_path = relative_corpus_path(root, file);
        let bytes =
            fs::read(file).map_err(|e| CorpusError::IoError(format!("{relative_path}: {e}")))?;
        let Ok(message) = parse_corpus_message_bytes(&bytes) else {
            continue;
        };

        let mut message_occurrences: BTreeMap<String, usize> = BTreeMap::new();
        record_fingerprint_message_shape(&message, &mut message_occurrences, &mut value_shapes);

        for (path, occurrences) in message_occurrences {
            let entry = cardinality.entry(path).or_default();
            entry.present_message_count = entry.present_message_count.saturating_add(1);
            entry.total_occurrences = entry.total_occurrences.saturating_add(occurrences);
            entry.max_per_message = entry.max_per_message.max(occurrences);
            entry.min_present_per_message = match entry.min_present_per_message {
                Some(current) => Some(current.min(occurrences)),
                None => Some(occurrences),
            };
        }
    }

    let mut cardinality: Vec<CorpusFieldCardinality> = cardinality
        .into_iter()
        .map(|(path, stats)| {
            let min_per_message = if stats.present_message_count < message_count {
                0
            } else {
                stats.min_present_per_message.unwrap_or_default()
            };
            CorpusFieldCardinality {
                path,
                min_per_message,
                max_per_message: stats.max_per_message,
                total_occurrences: stats.total_occurrences,
                message_count: stats.present_message_count,
            }
        })
        .collect();
    cardinality.sort_by(|left, right| compare_field_paths(&left.path, &right.path));

    let mut value_shape_stats: Vec<CorpusValueShapeStats> = value_shapes.into_values().collect();
    value_shape_stats.sort_by(|left, right| compare_field_paths(&left.path, &right.path));

    Ok((cardinality, value_shape_stats))
}

fn collect_fingerprint_details_from_messages(
    messages: &[CorpusMessageRef<'_>],
    message_count: usize,
) -> (Vec<CorpusFieldCardinality>, Vec<CorpusValueShapeStats>) {
    let mut cardinality: BTreeMap<String, FieldCardinalityAccumulator> = BTreeMap::new();
    let mut value_shapes: BTreeMap<String, CorpusValueShapeStats> = BTreeMap::new();

    for message_ref in messages {
        let Ok(message) = parse_corpus_message_bytes(message_ref.bytes) else {
            continue;
        };

        let mut message_occurrences: BTreeMap<String, usize> = BTreeMap::new();
        record_fingerprint_message_shape(&message, &mut message_occurrences, &mut value_shapes);

        for (path, occurrences) in message_occurrences {
            let entry = cardinality.entry(path).or_default();
            entry.present_message_count = entry.present_message_count.saturating_add(1);
            entry.total_occurrences = entry.total_occurrences.saturating_add(occurrences);
            entry.max_per_message = entry.max_per_message.max(occurrences);
            entry.min_present_per_message = match entry.min_present_per_message {
                Some(current) => Some(current.min(occurrences)),
                None => Some(occurrences),
            };
        }
    }

    let mut cardinality: Vec<CorpusFieldCardinality> = cardinality
        .into_iter()
        .map(|(path, stats)| {
            let min_per_message = if stats.present_message_count < message_count {
                0
            } else {
                stats.min_present_per_message.unwrap_or_default()
            };
            CorpusFieldCardinality {
                path,
                min_per_message,
                max_per_message: stats.max_per_message,
                total_occurrences: stats.total_occurrences,
                message_count: stats.present_message_count,
            }
        })
        .collect();
    cardinality.sort_by(|left, right| compare_field_paths(&left.path, &right.path));

    let mut value_shape_stats: Vec<CorpusValueShapeStats> = value_shapes.into_values().collect();
    value_shape_stats.sort_by(|left, right| compare_field_paths(&left.path, &right.path));

    (cardinality, value_shape_stats)
}

fn record_message_shape(
    message: &Message,
    segment_counts: &mut BTreeMap<String, usize>,
    field_message_counts: &mut BTreeMap<String, usize>,
    field_occurrence_counts: &mut BTreeMap<String, usize>,
) {
    let mut message_field_paths = BTreeSet::new();

    for segment in &message.segments {
        let segment_id = segment.id_str().to_string();
        increment_count(segment_counts, segment_id.clone());

        for (field_index, field) in segment.fields.iter().enumerate() {
            if !field_is_present(field) {
                continue;
            }

            let display_index = if segment_id == "MSH" {
                field_index.saturating_add(2)
            } else {
                field_index.saturating_add(1)
            };
            let path = format!("{segment_id}.{display_index}");
            increment_count(field_occurrence_counts, path.clone());
            message_field_paths.insert(path);
        }
    }

    for path in message_field_paths {
        increment_count(field_message_counts, path);
    }
}

#[derive(Default)]
struct FieldCardinalityAccumulator {
    present_message_count: usize,
    total_occurrences: usize,
    max_per_message: usize,
    min_present_per_message: Option<usize>,
}

fn record_fingerprint_message_shape(
    message: &Message,
    message_occurrences: &mut BTreeMap<String, usize>,
    value_shapes: &mut BTreeMap<String, CorpusValueShapeStats>,
) {
    for segment in &message.segments {
        let segment_id = segment.id_str().to_string();

        for (field_index, field) in segment.fields.iter().enumerate() {
            if !field_is_present(field) {
                continue;
            }

            let display_index = if segment_id == "MSH" {
                field_index.saturating_add(2)
            } else {
                field_index.saturating_add(1)
            };
            let path = format!("{segment_id}.{display_index}");
            increment_count(message_occurrences, path.clone());
            record_value_shape(value_shapes, &path, field);
        }
    }
}

fn field_is_present(field: &Field) -> bool {
    field.reps.iter().any(|rep| {
        rep.comps.iter().any(|comp| {
            comp.subs.iter().any(|atom| match atom {
                Atom::Text(text) => !text.is_empty(),
                Atom::Null => true,
            })
        })
    })
}

#[derive(Clone, Copy)]
enum ValueShape {
    Coded,
    Timestamp,
    Numeric,
    Null,
    Text,
}

fn record_value_shape(
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
    field
        .reps
        .iter()
        .filter_map(repetition_value_shape)
        .collect()
}

fn repetition_value_shape(rep: &crate::model::Rep) -> Option<ValueShape> {
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

pub(super) fn compare_field_paths(left: &str, right: &str) -> Ordering {
    let (left_segment, left_index) = split_field_path(left);
    let (right_segment, right_index) = split_field_path(right);

    left_segment
        .cmp(right_segment)
        .then(left_index.cmp(&right_index))
        .then(left.cmp(right))
}

fn split_field_path(path: &str) -> (&str, usize) {
    let Some((segment, field)) = path.split_once('.') else {
        return (path, usize::MAX);
    };
    let index = field.parse::<usize>().unwrap_or(usize::MAX);
    (segment, index)
}

fn increment_count(counts: &mut BTreeMap<String, usize>, value: String) {
    let count = counts.entry(value).or_insert(0);
    *count = count.saturating_add(1);
}

fn counts_to_vec(counts: BTreeMap<String, usize>) -> Vec<CorpusCount> {
    counts
        .into_iter()
        .map(|(value, count)| CorpusCount { value, count })
        .collect()
}

/// Extract message type from a message's MSH.9 field
pub fn extract_message_type(message: &Message) -> String {
    // Find MSH segment
    for segment in &message.segments {
        if &segment.id == b"MSH" {
            // MSH.9 is at index 8 (0-indexed: field 9 - 1 for skipping MSH-1/MSH-2)
            if let Some(field) = segment.fields.get(7)
                && let Some(rep) = field.reps.first()
                && !rep.comps.is_empty()
            {
                // Build the message type from components
                let parts: Vec<String> = rep
                    .comps
                    .iter()
                    .filter_map(|c| match c.subs.first() {
                        Some(Atom::Text(t)) => Some(t.clone()),
                        _ => None,
                    })
                    .collect();
                return parts.join("^");
            }
        }
    }
    "UNKNOWN".to_string()
}

#[cfg(test)]
mod summary_tests {
    #![expect(
        clippy::panic,
        reason = "Corpus summary tests fail explicitly on test setup errors."
    )]

    use super::*;
    use std::path::{Path, PathBuf};

    const ADT_A01: &str = "MSH|^~\\&|SENDAPP|SENDFAC|RECVAPP|RECVFAC|202605080101||ADT^A01|CTRL123|P|2.5\rPID|1||123456^^^HOSP^MR||Doe^John||19700101|M";
    const ORU_R01: &str = "MSH|^~\\&|LAB|LAB|EHR|HOSP|202605080101||ORU^R01|CTRL456|P|2.5\rPID|1||123456^^^HOSP^MR||Doe^John||19700101|M\rOBR|1|ORD1|FILL1|CBC^Complete Blood Count\rOBX|1|NM|718-7^Hemoglobin||13.2|g/dL";

    fn write_message(path: &Path, contents: &str) {
        let result = fs::write(path, contents);
        assert!(result.is_ok(), "test message should be written: {result:?}");
    }

    fn write_message_bytes(path: &Path, contents: &[u8]) {
        let result = fs::write(path, contents);
        assert!(result.is_ok(), "test message should be written: {result:?}");
    }

    fn dirty_real_world_fixture_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test_data/dirty-real-world")
    }

    fn normalize_fixture_segments(bytes: &[u8]) -> Vec<u8> {
        String::from_utf8_lossy(bytes)
            .replace("\r\n", "\n")
            .replace('\n', "\r")
            .into_bytes()
    }

    fn materialize_dirty_corpus_dir(category: &str, target: &Path) {
        let source = dirty_real_world_fixture_root().join(category);
        let result = fs::create_dir_all(target);
        assert!(
            result.is_ok(),
            "fixture target should be created: {result:?}"
        );

        let Ok(entries) = fs::read_dir(&source) else {
            panic!("dirty fixture category should be readable: {source:?}");
        };

        for entry in entries {
            let Ok(entry) = entry else {
                panic!("dirty fixture entry should be readable");
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                panic!("dirty fixture file should be readable: {path:?}");
            };
            let Some(file_name) = path.file_name() else {
                panic!("dirty fixture file should have a name: {path:?}");
            };
            write_message_bytes(&target.join(file_name), &normalize_fixture_segments(&bytes));
        }
    }

    fn add_generated_mllp_fixtures(target: &Path) {
        let source = dirty_real_world_fixture_root().join("sources/mllp-source.hl7");
        let Ok(bytes) = fs::read(&source) else {
            panic!("MLLP source fixture should be readable: {source:?}");
        };
        let normalized = normalize_fixture_segments(&bytes);
        let wrapped = crate::wrap_mllp(&normalized);
        write_message_bytes(&target.join("mllp-framed.hl7"), &wrapped);
        let mut truncated = wrapped;
        let _ = truncated.pop();
        write_message_bytes(&target.join("mllp-truncated.hl7"), &truncated);
    }

    #[test]
    fn summarize_corpus_path_counts_messages_segments_and_fields() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("test temp dir should be created");
        };
        write_message(&dir.path().join("adt.hl7"), ADT_A01);
        write_message(&dir.path().join("oru.hl7"), ORU_R01);

        let Ok(summary) = summarize_corpus_path(dir.path()) else {
            panic!("corpus should summarize");
        };

        assert_eq!(summary.file_count, 2);
        assert_eq!(summary.message_count, 2);
        assert_eq!(summary.parse_error_count, 0);
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ADT^A01" && count.count == 1)
        );
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ORU^R01" && count.count == 1)
        );
        assert!(
            summary
                .segments
                .iter()
                .any(|count| count.value == "PID" && count.count == 2)
        );
        assert!(
            summary
                .field_presence
                .iter()
                .any(|field| field.path == "PID.3" && field.message_count == 2)
        );

        let summary_v2 = summary.to_v2("hl7v2", "1.3.0");
        assert_eq!(summary_v2.schema_version, "2");
        assert_eq!(summary_v2.tool_name, "hl7v2");
        assert_eq!(summary_v2.tool_version, "1.3.0");
        assert_eq!(summary_v2.summary.file_count, 2);
    }

    #[test]
    fn summarize_corpus_path_records_parse_failures() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("test temp dir should be created");
        };
        write_message(&dir.path().join("valid.hl7"), ADT_A01);
        write_message(&dir.path().join("invalid.hl7"), "not an hl7 message");

        let Ok(summary) = summarize_corpus_path(dir.path()) else {
            panic!("corpus should summarize");
        };

        assert_eq!(summary.file_count, 2);
        assert_eq!(summary.message_count, 1);
        assert_eq!(summary.parse_error_count, 1);
        assert_eq!(
            summary
                .parse_errors
                .first()
                .map(|failure| failure.path.as_str()),
            Some("invalid.hl7")
        );
    }

    #[test]
    fn summarize_corpus_messages_uses_message_ids_for_parse_failures() {
        let messages = [
            CorpusMessageRef::new("adt-1", ADT_A01.as_bytes()),
            CorpusMessageRef::new("bad-1", b"not an hl7 message"),
        ];

        let summary = summarize_corpus_messages("<inline-corpus>", &messages);

        assert_eq!(summary.root, "<inline-corpus>");
        assert_eq!(summary.file_count, 2);
        assert_eq!(summary.message_count, 1);
        assert_eq!(summary.parse_error_count, 1);
        assert_eq!(
            summary
                .parse_errors
                .first()
                .map(|failure| failure.path.as_str()),
            Some("bad-1")
        );
        let Some(parse_failure) = summary.parse_errors.first() else {
            panic!("parse failure should be recorded");
        };
        assert!(!parse_failure.error.contains("not an hl7 message"));
    }

    #[test]
    fn dirty_real_world_corpus_produces_safe_summary_fingerprint_and_diff() {
        let Ok(before) = tempfile::tempdir() else {
            panic!("before temp dir should be created");
        };
        let Ok(after) = tempfile::tempdir() else {
            panic!("after temp dir should be created");
        };

        materialize_dirty_corpus_dir("before", before.path());
        materialize_dirty_corpus_dir("after", after.path());
        add_generated_mllp_fixtures(after.path());

        let Ok(summary) = summarize_corpus_path(after.path()) else {
            panic!("dirty corpus should summarize");
        };
        let Ok(fingerprint) = fingerprint_corpus_path(after.path()) else {
            panic!("dirty corpus should fingerprint");
        };
        let Ok(diff) = diff_corpus_paths(before.path(), after.path()) else {
            panic!("dirty corpus should diff");
        };

        assert_eq!(summary.file_count, 11);
        assert_eq!(summary.message_count, 8);
        assert_eq!(summary.parse_error_count, 3);
        assert!(summary.total_bytes > 1_000);
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ADT^A01" && count.count == 2)
        );
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ADT^A08" && count.count == 1)
        );
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ADT^A04" && count.count == 1)
        );
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ADT^A03" && count.count == 1)
        );
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ADT^A31" && count.count == 1)
        );
        assert!(
            summary
                .message_types
                .iter()
                .any(|count| count.value == "ORU^R01" && count.count == 2)
        );
        assert!(
            summary
                .segments
                .iter()
                .any(|count| count.value == "ZPV" && count.count == 1)
        );
        assert!(
            summary
                .segments
                .iter()
                .any(|count| count.value == "ZSB" && count.count == 1)
        );
        assert!(
            summary
                .segments
                .iter()
                .any(|count| count.value == "OBX" && count.count == 22)
        );
        assert!(
            summary
                .segments
                .iter()
                .any(|count| count.value == "NTE" && count.count == 1)
        );
        assert!(
            summary
                .parse_errors
                .iter()
                .any(|failure| failure.path == "malformed-delimiters.hl7")
        );
        assert!(
            summary
                .parse_errors
                .iter()
                .any(|failure| failure.path == "partial-batch.hl7")
        );
        assert!(
            summary
                .parse_errors
                .iter()
                .any(|failure| failure.path == "mllp-truncated.hl7")
        );
        assert!(
            summary
                .parse_errors
                .iter()
                .all(|failure| !failure.error.contains("MRN-DIRTY"))
        );

        assert_eq!(fingerprint.file_count, 11);
        assert_eq!(fingerprint.message_count, 8);
        assert_eq!(fingerprint.parse_error_count, 3);
        assert!(
            fingerprint
                .field_cardinality
                .iter()
                .any(|field| field.path == "OBX.5"
                    && field.max_per_message == 20
                    && field.total_occurrences == 22)
        );
        assert!(
            fingerprint
                .field_cardinality
                .iter()
                .any(|field| field.path == "ZPV.1" && field.total_occurrences == 1)
        );
        assert!(
            fingerprint
                .field_cardinality
                .iter()
                .any(|field| field.path == "MSH.3" && field.total_occurrences == 8)
        );
        assert!(
            fingerprint
                .field_cardinality
                .iter()
                .any(|field| field.path == "ZSB.1" && field.total_occurrences == 1)
        );
        assert!(
            fingerprint
                .value_shape_stats
                .iter()
                .any(|shape| shape.path == "PID.7" && shape.numeric_count >= 1)
        );
        assert!(
            fingerprint
                .value_shape_stats
                .iter()
                .any(|shape| shape.path == "PID.3" && shape.text_count >= 1)
        );
        assert!(
            fingerprint
                .value_shape_stats
                .iter()
                .any(|shape| shape.path == "OBX.5"
                    && shape.null_count == 1
                    && shape.text_count >= 1)
        );

        assert_eq!(diff.file_count.before, 2);
        assert_eq!(diff.file_count.after, 11);
        assert_eq!(diff.message_count.delta, 6);
        assert_eq!(diff.parse_error_count.delta, 3);
        assert!(
            diff.field_cardinality
                .iter()
                .any(|field| field.path == "OBX.5"
                    && field.max_per_message_delta == 15
                    && field.total_occurrences_delta == 17)
        );
        assert!(
            diff.value_shape_stats
                .iter()
                .any(|shape| shape.path == "OBX.5"
                    && shape.null_count.delta == 1
                    && shape.text_count.delta >= 1)
        );
    }

    #[test]
    fn fingerprint_corpus_messages_reports_shape_and_diffable_output() {
        let before_messages = [CorpusMessageRef::new("adt-1", ADT_A01.as_bytes())];
        let after_messages = [
            CorpusMessageRef::new("adt-1", ADT_A01.as_bytes()),
            CorpusMessageRef::new("oru-1", ORU_R01.as_bytes()),
        ];

        let before = fingerprint_corpus_messages("<inline-before>", &before_messages);
        let after = fingerprint_corpus_messages("<inline-after>", &after_messages);
        let diff = diff_corpus_fingerprints(&before, &after);

        assert_eq!(before.root, "<inline-before>");
        assert_eq!(after.root, "<inline-after>");
        assert_eq!(before.message_count, 1);
        assert_eq!(after.message_count, 2);
        assert_eq!(diff.new_message_types, vec!["ORU^R01".to_string()]);
        assert!(
            after
                .field_cardinality
                .iter()
                .any(|field| field.path == "OBX.5"
                    && field.min_per_message == 0
                    && field.max_per_message == 1)
        );
    }

    #[test]
    fn diff_corpus_paths_reports_count_deltas() {
        let Ok(before) = tempfile::tempdir() else {
            panic!("before temp dir should be created");
        };
        let Ok(after) = tempfile::tempdir() else {
            panic!("after temp dir should be created");
        };
        write_message(&before.path().join("adt.hl7"), ADT_A01);
        write_message(&after.path().join("adt.hl7"), ADT_A01);
        write_message(&after.path().join("oru.hl7"), ORU_R01);

        let Ok(diff) = diff_corpus_paths(before.path(), after.path()) else {
            panic!("corpus diff should be created");
        };

        assert_eq!(diff.file_count.before, 1);
        assert_eq!(diff.file_count.after, 2);
        assert_eq!(diff.file_count.delta, 1);
        assert_eq!(diff.message_count.delta, 1);
        assert!(
            diff.message_type_counts
                .iter()
                .any(|count| count.value == "ORU^R01" && count.before == 0 && count.after == 1)
        );
        assert_eq!(diff.new_message_types, vec!["ORU^R01".to_string()]);
        assert!(
            diff.segment_counts
                .iter()
                .any(|count| count.value == "OBX" && count.before == 0 && count.after == 1)
        );
        assert!(diff.new_segments.iter().any(|segment| segment == "OBX"));
        assert!(
            diff.field_presence
                .iter()
                .any(|field| field.path == "OBX.5" && field.message_count_delta == 1)
        );
        assert!(
            diff.field_cardinality
                .iter()
                .any(|field| field.path == "OBX.5"
                    && field.max_per_message_delta == 1
                    && field.total_occurrences_delta == 1)
        );
        assert!(
            diff.value_shape_stats
                .iter()
                .any(|shape| shape.path == "OBX.5" && shape.numeric_count.delta == 1)
        );

        let diff_v2 = diff.to_v2("hl7v2");
        assert_eq!(diff_v2.schema_version, "2");
        assert_eq!(diff_v2.tool_name, "hl7v2");
        assert_eq!(diff_v2.report.diff_version, "1");
    }

    #[test]
    fn fingerprint_corpus_path_reports_shape_and_cardinality() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("test temp dir should be created");
        };
        write_message(&dir.path().join("adt.hl7"), ADT_A01);
        write_message(&dir.path().join("oru.hl7"), ORU_R01);
        write_message(&dir.path().join("invalid.hl7"), "not an hl7 message");

        let Ok(fingerprint) = fingerprint_corpus_path(dir.path()) else {
            panic!("corpus should fingerprint");
        };

        assert_eq!(fingerprint.fingerprint_version, "1");
        assert_eq!(fingerprint.file_count, 3);
        assert_eq!(fingerprint.message_count, 2);
        assert_eq!(fingerprint.parse_error_count, 1);
        assert!(
            fingerprint
                .message_type_counts
                .iter()
                .any(|count| count.value == "ORU^R01" && count.count == 1)
        );
        assert!(
            fingerprint
                .field_cardinality
                .iter()
                .any(|field| field.path == "OBX.5"
                    && field.min_per_message == 0
                    && field.max_per_message == 1)
        );
        assert!(
            fingerprint
                .value_shape_stats
                .iter()
                .any(|shape| shape.path == "OBX.5" && shape.numeric_count == 1)
        );

        let fingerprint_v2 = fingerprint.to_v2("hl7v2");
        assert_eq!(fingerprint_v2.schema_version, "2");
        assert_eq!(fingerprint_v2.tool_name, "hl7v2");
        assert_eq!(fingerprint_v2.fingerprint.fingerprint_version, "1");
    }
}
