use super::{
    Batch, BatchError, BatchInfo, BatchType, FileBatch, info::extract_batch_info,
    segment::parse_segment, segment_prefix,
};
use crate::{model::Message, parser::parse};

pub fn parse_batch(data: &[u8]) -> Result<FileBatch, BatchError> {
    let text = std::str::from_utf8(data)
        .map_err(|_err| BatchError::InvalidStructure("Invalid UTF-8 data".to_string()))?;
    let lines: Vec<&str> = text.split(['\r', '\n']).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return Err(BatchError::InvalidStructure("Empty batch data".to_string()));
    }
    let Some(first_line) = lines.first().copied() else {
        return Err(BatchError::InvalidStructure("Empty batch data".to_string()));
    };

    if first_line.starts_with("FHS") {
        parse_file_batch(&lines)
    } else if first_line.starts_with("BHS") {
        let batch = parse_single_batch(&lines)?;
        let mut file_batch = FileBatch::new();
        file_batch.info.batch_type = BatchType::Single;
        file_batch.info.field_separator = batch.info.field_separator;
        file_batch.info.encoding_characters = batch.info.encoding_characters.clone();
        file_batch.info.sending_application = batch.info.sending_application.clone();
        file_batch.info.sending_facility = batch.info.sending_facility.clone();
        file_batch.info.receiving_application = batch.info.receiving_application.clone();
        file_batch.info.receiving_facility = batch.info.receiving_facility.clone();
        file_batch.info.security = batch.info.security.clone();
        file_batch.info.batch_name = batch.info.batch_name.clone();
        file_batch.info.batch_comment = batch.info.batch_comment.clone();
        file_batch.info.message_count = batch.info.message_count;
        file_batch.info.trailer_comment = batch.info.trailer_comment.clone();
        file_batch.add_batch(batch);
        Ok(file_batch)
    } else if first_line.starts_with("MSH") {
        let messages = parse_messages(&lines)?;
        let batch = Batch {
            header: None,
            messages,
            trailer: None,
            info: BatchInfo::default(),
        };
        let mut file_batch = FileBatch::new();
        file_batch.add_batch(batch);
        Ok(file_batch)
    } else {
        Err(BatchError::InvalidStructure(format!(
            "Unknown first segment: {}",
            segment_prefix(first_line)
        )))
    }
}

fn parse_file_batch(lines: &[&str]) -> Result<FileBatch, BatchError> {
    let mut file_batch = FileBatch::new();
    let mut current_batch_lines: Vec<&str> = Vec::new();
    let mut current_message_lines: Vec<&str> = Vec::new();
    let mut in_batch = false;
    let mut has_fhs = false;
    for line in lines {
        if line.starts_with("FHS") {
            add_unwrapped_message_batch(&mut file_batch, &current_message_lines)?;
            current_message_lines.clear();
            has_fhs = true;
            file_batch.header = Some(parse_segment(line)?);
            let info = extract_batch_info(line, "FHS")?;
            file_batch.info.encoding_characters = info.encoding_characters;
            file_batch.info.sending_application = info.sending_application;
            file_batch.info.sending_facility = info.sending_facility;
            file_batch.info.receiving_application = info.receiving_application;
            file_batch.info.receiving_facility = info.receiving_facility;
            file_batch.info.file_creation_time = info.file_creation_time;
            file_batch.info.security = info.security;
            file_batch.info.field_separator = info.field_separator;
            file_batch.info.batch_name = info.batch_name;
            file_batch.info.batch_comment = info.batch_comment;
        } else if line.starts_with("FTS") {
            add_unwrapped_message_batch(&mut file_batch, &current_message_lines)?;
            current_message_lines.clear();
            file_batch.trailer = Some(parse_segment(line)?);
            let info = extract_batch_info(line, "FTS")?;
            file_batch.info.message_count = info.message_count;
            file_batch.info.trailer_comment = info.trailer_comment;
        } else if line.starts_with("BHS") {
            add_unwrapped_message_batch(&mut file_batch, &current_message_lines)?;
            current_message_lines.clear();
            in_batch = true;
            current_batch_lines.push(line);
        } else if line.starts_with("BTS") {
            current_batch_lines.push(line);
            let batch = parse_single_batch(&current_batch_lines)?;
            file_batch.add_batch(batch);
            current_batch_lines.clear();
            in_batch = false;
        } else if in_batch {
            current_batch_lines.push(line);
        } else if line.starts_with("MSH") {
            add_unwrapped_message_batch(&mut file_batch, &current_message_lines)?;
            current_message_lines.clear();
            current_message_lines.push(line);
        } else if !current_message_lines.is_empty() {
            current_message_lines.push(line);
        }
    }
    add_unwrapped_message_batch(&mut file_batch, &current_message_lines)?;
    if !has_fhs {
        return Err(BatchError::MissingSegment("FHS".to_string()));
    }
    let actual_message_count = file_batch.total_message_count();
    match file_batch.info.message_count {
        Some(expected) if expected != actual_message_count => Err(BatchError::CountMismatch {
            expected,
            actual: actual_message_count,
        }),
        Some(_) => Ok(file_batch),
        None => {
            file_batch.info.message_count = Some(actual_message_count);
            Ok(file_batch)
        }
    }
}

fn add_unwrapped_message_batch(
    file_batch: &mut FileBatch,
    message_lines: &[&str],
) -> Result<(), BatchError> {
    if message_lines.is_empty() {
        return Ok(());
    }
    let messages = parse_messages(message_lines)?;
    file_batch.add_batch(Batch {
        header: None,
        messages,
        trailer: None,
        info: BatchInfo::default(),
    });
    Ok(())
}

fn parse_single_batch(lines: &[&str]) -> Result<Batch, BatchError> {
    let mut batch = Batch::new();
    let mut message_lines: Vec<&str> = Vec::new();
    let mut has_bhs = false;
    let mut has_bts = false;
    for line in lines {
        if line.starts_with("BHS") {
            has_bhs = true;
            batch.header = Some(parse_segment(line)?);
            batch.info = extract_batch_info(line, "BHS")?;
        } else if line.starts_with("BTS") {
            has_bts = true;
            batch.trailer = Some(parse_segment(line)?);
            let info = extract_batch_info(line, "BTS")?;
            batch.info.message_count = info.message_count;
            batch.info.trailer_comment = info.trailer_comment;
        } else if line.starts_with("MSH") {
            if !message_lines.is_empty() {
                let msg_text = message_lines.join("\r");
                let msg = parse(msg_text.as_bytes())?;
                batch.add_message(msg);
                message_lines.clear();
            }
            message_lines.push(line);
        } else {
            message_lines.push(line);
        }
    }
    if !message_lines.is_empty() {
        let msg_text = message_lines.join("\r");
        let msg = parse(msg_text.as_bytes())?;
        batch.add_message(msg);
    }
    if !has_bhs && (has_bts || !batch.messages.is_empty()) {
        return Err(BatchError::MissingSegment("BHS".to_string()));
    }
    if !has_bts && (has_bhs || !batch.messages.is_empty()) {
        return Err(BatchError::MissingSegment("BTS".to_string()));
    }
    if batch.info.message_count.is_none() {
        batch.info.message_count = Some(batch.message_count());
    }
    if let Some(expected) = batch.info.message_count
        && expected != batch.message_count()
    {
        return Err(BatchError::CountMismatch {
            expected,
            actual: batch.message_count(),
        });
    }
    Ok(batch)
}

fn parse_messages(lines: &[&str]) -> Result<Vec<Message>, BatchError> {
    let mut messages = Vec::new();
    let mut message_lines: Vec<&str> = Vec::new();
    for line in lines {
        if line.starts_with("MSH") && !message_lines.is_empty() {
            let msg_text = message_lines.join("\r");
            let msg = parse(msg_text.as_bytes())?;
            messages.push(msg);
            message_lines.clear();
        }
        message_lines.push(line);
    }
    if !message_lines.is_empty() {
        let msg_text = message_lines.join("\r");
        let msg = parse(msg_text.as_bytes())?;
        messages.push(msg);
    }
    Ok(messages)
}
