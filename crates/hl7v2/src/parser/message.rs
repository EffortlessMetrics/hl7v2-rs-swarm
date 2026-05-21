//! Top-level HL7 v2 message parsing entry points.

#![expect(
    clippy::map_err_ignore,
    clippy::missing_errors_doc,
    reason = "parser entry points preserve existing error behavior while parser responsibilities are split into SRP submodules"
)]

use crate::model::{Error, Message};

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
    let text = decode::utf8(bytes)?;
    let lines = segment_lines(text);
    let first_line = validate::first_segment(&lines)?;
    let delims = delimiters::from_msh(first_line)?;
    let segments = segments::parse_all(&lines, &delims)?;

    Ok(assemble::message(delims, segments))
}

mod decode {
    use crate::model::Error;

    pub(super) fn utf8(bytes: &[u8]) -> Result<&str, Error> {
        std::str::from_utf8(bytes).map_err(|_| Error::InvalidCharset)
    }
}

mod validate {
    use crate::model::Error;

    pub(super) fn first_segment<'a>(lines: &'a [&str]) -> Result<&'a str, Error> {
        if lines.is_empty() {
            std::hint::cold_path();
            return Err(Error::InvalidSegmentId);
        }

        let Some(first_line) = lines.first() else {
            std::hint::cold_path();
            return Err(Error::InvalidSegmentId);
        };

        if !first_line.starts_with("MSH") {
            std::hint::cold_path();
            return Err(Error::InvalidSegmentId);
        }

        Ok(first_line)
    }
}

mod delimiters {
    use crate::model::{Delims, Error};

    pub(super) fn from_msh(first_line: &str) -> Result<Delims, Error> {
        Delims::parse_from_msh(first_line).map_err(|e| Error::ParseError {
            segment_id: "MSH".to_string(),
            field_index: 0,
            source: Box::new(e),
        })
    }
}

mod segments {
    use crate::model::{Delims, Error, Segment};
    use crate::parser::segment::parse_segment;

    pub(super) fn parse_all(lines: &[&str], delims: &Delims) -> Result<Vec<Segment>, Error> {
        let mut segments = Vec::with_capacity(lines.len());
        for line in lines {
            let segment = parse_segment(line, delims).map_err(|e| Error::ParseError {
                segment_id: line.get(..3).unwrap_or(line).to_string(),
                field_index: 0,
                source: Box::new(e),
            })?;
            segments.push(segment);
        }

        Ok(segments)
    }
}

mod assemble {
    use crate::model::{Delims, Message, Segment};
    use crate::parser::charset::extract_charsets;

    pub(super) fn message(delims: Delims, segments: Vec<Segment>) -> Message {
        let charsets = extract_charsets(&segments);

        Message {
            delims,
            segments,
            charsets,
        }
    }
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
