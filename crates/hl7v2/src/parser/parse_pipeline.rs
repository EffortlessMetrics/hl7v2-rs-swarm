use crate::model::{Delims, Error, Message, Segment};

use super::charset::extract_charsets;
use super::message::segment_lines;
use super::segment::parse_segment;

pub(super) fn parse_from_bytes(bytes: &[u8]) -> Result<Message, Error> {
    let text = decode_utf8(bytes)?;
    let lines = segment_lines(text);

    validate_first_segment(&lines)?;
    let delims = parse_delimiters(lines[0])?;
    let segments = parse_segments(&lines, &delims)?;
    let charsets = extract_charsets(&segments);

    Ok(Message {
        delims,
        segments,
        charsets,
    })
}

fn decode_utf8(bytes: &[u8]) -> Result<&str, Error> {
    std::str::from_utf8(bytes).map_err(|_| Error::InvalidCharset)
}

fn validate_first_segment(lines: &[&str]) -> Result<(), Error> {
    let Some(first_line) = lines.first() else {
        std::hint::cold_path();
        return Err(Error::InvalidSegmentId);
    };

    if !first_line.starts_with("MSH") {
        std::hint::cold_path();
        return Err(Error::InvalidSegmentId);
    }

    Ok(())
}

fn parse_delimiters(first_line: &str) -> Result<Delims, Error> {
    Delims::parse_from_msh(first_line).map_err(|e| Error::ParseError {
        segment_id: "MSH".to_string(),
        field_index: 0,
        source: Box::new(e),
    })
}

fn parse_segments(lines: &[&str], delims: &Delims) -> Result<Vec<Segment>, Error> {
    let mut segments = Vec::new();
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
