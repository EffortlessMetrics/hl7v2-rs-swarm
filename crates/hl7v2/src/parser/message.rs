//! Top-level HL7 v2 message parsing entry points.

#![expect(
    clippy::map_err_ignore,
    clippy::missing_errors_doc,
    reason = "parser entry points preserve existing error behavior while parser responsibilities are split into SRP submodules"
)]

use crate::model::{Error, Message};

use super::parse_pipeline::parse_from_bytes;

/// Parse HL7 v2 message from bytes.
///
/// This is the primary entry point for parsing HL7 messages.
///
/// # Arguments
///
/// * `bytes` - The raw HL7 message bytes
///
/// # Returns
///
/// The parsed `Message`, or an error if parsing fails
///
/// # Example
///
/// ```
/// use hl7v2::parser::parse;
///
/// let hl7 = b"MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|20250128152312||ADT^A01|ABC123|P|2.5.1\rPID|1||123456^^^HOSP^MR||Doe^John\r";
/// let message = parse(hl7).unwrap();
/// assert_eq!(message.segments.len(), 2);
/// ```
pub fn parse(bytes: &[u8]) -> Result<Message, Error> {
    parse_from_bytes(bytes)
}

/// Parse HL7 v2 message from MLLP framed bytes.
///
/// This function first removes the MLLP framing and then parses the message.
///
/// # Arguments
///
/// * `bytes` - The MLLP-framed HL7 message bytes
///
/// # Returns
///
/// The parsed `Message`, or an error if parsing fails
///
/// # Example
///
/// ```
/// use hl7v2::parser::parse_mllp;
/// use hl7v2::wrap_mllp;
///
/// let hl7 = b"MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|20250128152312||ADT^A01|ABC123|P|2.5.1\r";
/// let framed = wrap_mllp(hl7);
/// let message = parse_mllp(&framed).unwrap();
/// assert_eq!(message.segments.len(), 1);
/// ```
pub fn parse_mllp(bytes: &[u8]) -> Result<Message, Error> {
    let hl7_content =
        crate::transport::mllp::unwrap_mllp(bytes).map_err(|e| Error::Framing(e.to_string()))?;
    parse(hl7_content)
}

#[derive(Clone, Copy)]
pub(super) struct SegmentLine<'a> {
    pub(super) text: &'a str,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) fn segment_lines(text: &str) -> Vec<&str> {
    segment_line_spans(text)
        .into_iter()
        .map(|line| line.text)
        .collect()
}

pub(super) fn segment_line_spans(text: &str) -> Vec<SegmentLine<'_>> {
    let mut lines = Vec::new();
    let mut offset: usize = 0;

    for line in text.split('\r') {
        let raw_start = offset;
        let raw_end = raw_start.checked_add(line.len()).unwrap_or(text.len());
        offset = raw_end.checked_add(1).unwrap_or(raw_end);

        let (stripped, start) = line
            .strip_prefix('\n')
            .map_or((line, raw_start), |stripped| {
                (stripped, raw_start.checked_add(1).unwrap_or(raw_start))
            });
        if stripped.is_empty() {
            continue;
        }

        lines.push(SegmentLine {
            text: stripped,
            start,
            end: raw_end,
        });
    }

    lines
}
