use super::BatchError;

/// Type of batch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchType {
    Single,
    File,
}

/// Batch information extracted from header segments
#[derive(Debug, Clone, PartialEq)]
pub struct BatchInfo {
    pub batch_type: BatchType,
    pub field_separator: Option<char>,
    pub encoding_characters: Option<String>,
    pub sending_application: Option<String>,
    pub sending_facility: Option<String>,
    pub receiving_application: Option<String>,
    pub receiving_facility: Option<String>,
    pub file_creation_time: Option<String>,
    pub security: Option<String>,
    pub batch_name: Option<String>,
    pub batch_comment: Option<String>,
    pub message_count: Option<usize>,
    pub trailer_comment: Option<String>,
}

impl Default for BatchInfo {
    fn default() -> Self {
        Self {
            batch_type: BatchType::Single,
            field_separator: None,
            encoding_characters: None,
            sending_application: None,
            sending_facility: None,
            receiving_application: None,
            receiving_facility: None,
            file_creation_time: None,
            security: None,
            batch_name: None,
            batch_comment: None,
            message_count: None,
            trailer_comment: None,
        }
    }
}

pub(crate) fn extract_batch_info(line: &str, segment_type: &str) -> Result<BatchInfo, BatchError> {
    let mut info = BatchInfo::default();
    if line.len() < 4 {
        return Ok(info);
    }

    let field_sep = line.chars().nth(3).unwrap_or('|');
    info.field_separator = Some(field_sep);
    let fields: Vec<&str> = super::fields_after_separator(line)
        .split(field_sep)
        .collect();

    if segment_type == "FTS" || segment_type == "BTS" {
        info.message_count = fields.first().and_then(|s| s.parse::<usize>().ok());
        if let Some(comment) = fields.get(1) {
            info.trailer_comment = Some((*comment).to_string());
        }
        return Ok(info);
    }

    if let Some(v) = fields.first() {
        info.encoding_characters = Some((*v).to_string());
    }
    if let Some(v) = fields.get(1) {
        info.sending_application = Some((*v).to_string());
    }
    if let Some(v) = fields.get(2) {
        info.sending_facility = Some((*v).to_string());
    }
    if let Some(v) = fields.get(3) {
        info.receiving_application = Some((*v).to_string());
    }
    if let Some(v) = fields.get(4) {
        info.receiving_facility = Some((*v).to_string());
    }
    if let Some(v) = fields.get(5)
        && segment_type == "FHS"
    {
        info.file_creation_time = Some((*v).to_string());
    }
    if let Some(v) = fields.get(6) {
        info.security = Some((*v).to_string());
    }
    if let Some(v) = fields.get(8) {
        info.batch_name = Some((*v).to_string());
    }
    if let Some(v) = fields.get(9) {
        info.batch_comment = Some((*v).to_string());
    }

    Ok(info)
}
