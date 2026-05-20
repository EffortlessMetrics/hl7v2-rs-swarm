use super::*;
use crate::query::get;
use std::fmt::Debug;

fn message(control_id: &str) -> String {
    format!(
        "MSH|^~\\&|APP|FAC|RCV|RCVFAC|202605030101||ADT^A01|{control_id}|P|2.5.1\rPID|1||MRN^^^HOSP^MR||Doe^John"
    )
}

fn require_eq<T>(actual: T, expected: T, label: &str) -> Result<(), Box<dyn std::error::Error>>
where
    T: PartialEq + Debug,
{
    if actual == expected {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("{label}: expected {expected:?}, got {actual:?}")).into())
    }
}

fn require(condition: bool, message: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    if condition {
        Ok(())
    } else {
        Err(std::io::Error::other(message).into())
    }
}

#[test]
fn parse_batch_rejects_invalid_utf8_before_segment_processing()
-> Result<(), Box<dyn std::error::Error>> {
    let result = parse_batch(&[0xff, 0xfe, 0xfd]);
    require(
        matches!(result, Err(BatchError::InvalidStructure(message)) if message == "Invalid UTF-8 data"),
        "invalid UTF-8 should fail before segment processing",
    )?;
    Ok(())
}

#[test]
fn parse_batch_reports_unknown_first_segment_prefix() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_batch(b"ZZ\r");
    require(
        matches!(result, Err(BatchError::InvalidStructure(message)) if message == "Unknown first segment: ZZ"),
        "unknown first segment should report the prefix",
    )?;
    Ok(())
}

#[test]
fn parse_single_batch_preserves_header_and_trailer_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let data = format!(
        "BHS*:+\\&*SEND*SFAC*RECV*RFAC*202605030101*SEC**BATCH42*Nightly import\r{}\rBTS*1*done\r",
        message("CTRL1")
    );
    let batch = parse_batch(data.as_bytes())?;
    require_eq(batch.info.batch_type, BatchType::Single, "batch type")?;
    require_eq(batch.info.field_separator, Some('*'), "field separator")?;
    require_eq(
        batch.info.encoding_characters.as_deref(),
        Some(":+\\&"),
        "encoding characters",
    )?;
    require_eq(batch.info.message_count, Some(1), "message count")?;
    Ok(())
}

#[test]
fn parse_file_batch_collects_unwrapped_messages_separately()
-> Result<(), Box<dyn std::error::Error>> {
    let data = format!(
        "FHS|^~\\&|FILEAPP|FILEFAC|||202605030101\r{}\r{}\rFTS|2|complete\r",
        message("CTRL1"),
        message("CTRL2")
    );
    let batch = parse_batch(data.as_bytes())?;
    require_eq(batch.info.batch_type, BatchType::File, "batch type")?;
    require_eq(batch.total_message_count(), 2, "total message count")?;
    let first_message = batch
        .batches
        .first()
        .and_then(|batch| batch.messages.first())
        .ok_or_else(|| std::io::Error::other("missing first unwrapped message"))?;
    require_eq(
        get(first_message, "PID.5.1"),
        Some("Doe"),
        "first unwrapped message PID-5.1",
    )?;
    Ok(())
}

#[test]
fn parse_segment_preserves_empty_fields_with_custom_separator()
-> Result<(), Box<dyn std::error::Error>> {
    let segment = super::segment::parse_segment("BTS*2**comment")?;
    require_eq(segment.id, *b"BTS", "segment id")?;
    require_eq(
        segment
            .fields
            .get(1)
            .and_then(crate::model::Field::first_text),
        Some(""),
        "empty second field",
    )?;
    Ok(())
}

#[test]
fn fields_after_separator_handles_short_and_multibyte_segments() {
    assert_eq!(super::fields_after_separator("BHS|fields"), "fields");
    assert_eq!(super::fields_after_separator("MSH"), "");
    assert_eq!(super::fields_after_separator("AAAÅ|fields"), "");
}
