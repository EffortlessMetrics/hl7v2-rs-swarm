/// Application error type with specific error variants.
///
/// This enum provides detailed error information for different failure modes,
/// making it easier to diagnose issues and provide meaningful error responses.
#[derive(Debug)]
pub enum AppError {
    /// Message parsing error (malformed HL7, invalid structure, etc.)
    Parse(String),

    /// Profile loading error (YAML syntax, missing fields, etc.)
    ProfileLoad(String),

    /// Validation error (message does not conform to profile)
    Validation(String),

    /// Redaction policy or redaction application error
    Redaction(String),

    /// Bundle output is not configured on the server
    BundleOutputNotConfigured,

    /// Bundle output root is configured but not writable or available
    BundleOutputNotReady(String),

    /// Bundle request or write error
    Bundle(String),

    /// Bundle output already exists
    Conflict(String),

    /// Requested bundle id was not found under the configured output root
    BundleNotFound(String),

    /// Quarantine output is enabled but no path is configured
    QuarantineOutputNotConfigured,

    /// Quarantine output root is configured but not writable or available
    QuarantineOutputNotReady(String),

    /// Quarantine output write error
    Quarantine(String),

    /// Quarantine output already exists
    QuarantineConflict(String),

    /// Internal server error (unexpected failures)
    Internal(String),
}
