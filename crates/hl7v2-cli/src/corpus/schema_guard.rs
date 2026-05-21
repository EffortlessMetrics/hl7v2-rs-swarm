use crate::cli::ReportFormat;

pub(super) fn ensure_schema_format_support(
    schema_version: u8,
    format: &ReportFormat,
    error_message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if schema_version == 2 && *format == ReportFormat::Text {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error_message).into());
    }

    Ok(())
}
