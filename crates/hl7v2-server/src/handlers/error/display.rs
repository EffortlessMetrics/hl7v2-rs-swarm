use super::AppError;
use crate::PROFILE_LOAD_SAFE_MESSAGE;

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::ProfileLoad(_) => {
                write!(f, "Profile load error: {PROFILE_LOAD_SAFE_MESSAGE}")
            }
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::Redaction(msg) => write!(f, "Redaction error: {}", msg),
            AppError::BundleOutputNotConfigured => {
                write!(f, "Bundle output root is not configured")
            }
            AppError::BundleOutputNotReady(msg) => {
                write!(f, "Bundle output root is not ready: {}", msg)
            }
            AppError::Bundle(msg) => write!(f, "Bundle error: {}", msg),
            AppError::Conflict(msg) => write!(f, "Bundle conflict: {}", msg),
            AppError::BundleNotFound(msg) => write!(f, "Bundle not found: {}", msg),
            AppError::QuarantineOutputNotConfigured => {
                write!(f, "Quarantine output path is not configured")
            }
            AppError::QuarantineOutputNotReady(msg) => {
                write!(f, "Quarantine output root is not ready: {}", msg)
            }
            AppError::Quarantine(msg) => write!(f, "Quarantine error: {}", msg),
            AppError::QuarantineConflict(msg) => write!(f, "Quarantine conflict: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}
