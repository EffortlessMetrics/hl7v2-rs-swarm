//! HL7 v2 batch message handling (FHS/BHS/FTS/BTS).
//!
//! This crate provides batch processing for HL7 v2 messages, supporting:
//! - File Batch Header (FHS) and Trailer (FTS)
//! - Batch Header (BHS) and Trailer (BTS)
//! - Nested batch structures

mod info;
mod parse;
mod segment;

pub use info::{BatchInfo, BatchType};
pub use parse::parse_batch;
pub(crate) use segment::{fields_after_separator, segment_prefix};

use crate::model::{Error as ModelError, Message, Segment};
use thiserror::Error;

/// Error type for batch operations
#[derive(Debug, Error, Clone)]
pub enum BatchError {
    /// The batch structure does not match the expected HL7 format.
    #[error("Invalid batch structure: {0}")]
    InvalidStructure(String),

    /// A required segment is missing.
    #[error("Missing required segment: {0}")]
    MissingSegment(String),

    /// Found start and end batch markers that do not align.
    #[error("Mismatched batch headers/trailers")]
    MismatchedHeaders,

    /// General parsing error while reading batch input.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// The batch trailer count does not match observed messages.
    #[error("Count mismatch: expected {expected}, got {actual}")]
    CountMismatch { expected: usize, actual: usize },
}

impl From<ModelError> for BatchError {
    fn from(e: ModelError) -> Self {
        BatchError::ParseError(e.to_string())
    }
}

/// A single batch containing messages
#[derive(Debug, Clone, PartialEq)]
pub struct Batch {
    pub header: Option<Segment>,
    pub messages: Vec<Message>,
    pub trailer: Option<Segment>,
    pub info: BatchInfo,
}

impl Batch {
    pub fn new() -> Self {
        Self {
            header: None,
            messages: Vec::new(),
            trailer: None,
            info: BatchInfo::default(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn iter_messages(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter()
    }
}

impl Default for Batch {
    fn default() -> Self {
        Self::new()
    }
}

/// A file batch containing nested batches or messages
#[derive(Debug, Clone, PartialEq)]
pub struct FileBatch {
    pub header: Option<Segment>,
    pub batches: Vec<Batch>,
    pub trailer: Option<Segment>,
    pub info: BatchInfo,
}

impl FileBatch {
    pub fn new() -> Self {
        Self {
            header: None,
            batches: Vec::new(),
            trailer: None,
            info: BatchInfo {
                batch_type: BatchType::File,
                ..BatchInfo::default()
            },
        }
    }

    pub fn add_batch(&mut self, batch: Batch) {
        self.batches.push(batch);
    }

    pub fn total_message_count(&self) -> usize {
        self.batches.iter().map(Batch::message_count).sum()
    }

    pub fn iter_all_messages(&self) -> impl Iterator<Item = &Message> {
        self.batches.iter().flat_map(|b| b.messages.iter())
    }
}

impl Default for FileBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
