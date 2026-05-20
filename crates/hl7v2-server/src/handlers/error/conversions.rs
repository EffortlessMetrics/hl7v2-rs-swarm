use super::AppError;
use crate::PROFILE_LOAD_SAFE_MESSAGE;

impl From<crate::evidence::EvidenceBundleError> for AppError {
    fn from(error: crate::evidence::EvidenceBundleError) -> Self {
        match error {
            crate::evidence::EvidenceBundleError::InvalidRequest(message) => Self::Bundle(message),
            crate::evidence::EvidenceBundleError::Conflict(message) => Self::Conflict(message),
            crate::evidence::EvidenceBundleError::Io(message) => {
                Self::BundleOutputNotReady(message)
            }
        }
    }
}

impl AppError {
    pub(crate) fn quarantine_from_evidence_error(
        error: crate::evidence::EvidenceBundleError,
    ) -> Self {
        match error {
            crate::evidence::EvidenceBundleError::InvalidRequest(message) => {
                Self::Quarantine(message)
            }
            crate::evidence::EvidenceBundleError::Conflict(message) => {
                Self::QuarantineConflict(message)
            }
            crate::evidence::EvidenceBundleError::Io(message) => {
                Self::QuarantineOutputNotReady(message)
            }
        }
    }
}

impl From<hl7v2::Error> for AppError {
    fn from(err: hl7v2::Error) -> Self {
        AppError::Parse(err.to_string())
    }
}

impl From<hl7v2::conformance::profile::ProfileLoadError> for AppError {
    fn from(_err: hl7v2::conformance::profile::ProfileLoadError) -> Self {
        AppError::ProfileLoad(PROFILE_LOAD_SAFE_MESSAGE.to_string())
    }
}
