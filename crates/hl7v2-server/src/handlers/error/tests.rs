use super::*;
use crate::PROFILE_LOAD_SAFE_MESSAGE;
use crate::evidence::EvidenceBundleError;

#[test]
fn from_evidence_bundle_error_maps_each_variant_to_a_bundle_app_error() {
    let invalid: AppError = EvidenceBundleError::InvalidRequest("bad request".to_string()).into();
    assert!(
        matches!(&invalid, AppError::Bundle(m) if m == "bad request"),
        "got {invalid:?}"
    );

    let conflict: AppError = EvidenceBundleError::Conflict("dup".to_string()).into();
    assert!(
        matches!(&conflict, AppError::Conflict(m) if m == "dup"),
        "got {conflict:?}"
    );

    let io_err: AppError = EvidenceBundleError::Io("disk full".to_string()).into();
    assert!(
        matches!(&io_err, AppError::BundleOutputNotReady(m) if m == "disk full"),
        "got {io_err:?}"
    );
}

#[test]
fn quarantine_from_evidence_error_maps_each_variant_to_quarantine_app_error() {
    let invalid = AppError::quarantine_from_evidence_error(EvidenceBundleError::InvalidRequest(
        "bad request".to_string(),
    ));
    assert!(
        matches!(&invalid, AppError::Quarantine(m) if m == "bad request"),
        "got {invalid:?}"
    );

    let conflict = AppError::quarantine_from_evidence_error(EvidenceBundleError::Conflict(
        "exists".to_string(),
    ));
    assert!(
        matches!(&conflict, AppError::QuarantineConflict(m) if m == "exists"),
        "got {conflict:?}"
    );

    let io_err =
        AppError::quarantine_from_evidence_error(EvidenceBundleError::Io("io fail".to_string()));
    assert!(
        matches!(&io_err, AppError::QuarantineOutputNotReady(m) if m == "io fail"),
        "got {io_err:?}"
    );
}

#[test]
fn display_prefixes_each_variant_with_a_meaningful_label() {
    assert_eq!(
        AppError::Parse("oops".to_string()).to_string(),
        "Parse error: oops"
    );
    assert_eq!(
        AppError::Validation("bad".to_string()).to_string(),
        "Validation error: bad"
    );
    assert_eq!(
        AppError::Redaction("policy".to_string()).to_string(),
        "Redaction error: policy"
    );
    assert_eq!(
        AppError::Bundle("write".to_string()).to_string(),
        "Bundle error: write"
    );
    assert_eq!(
        AppError::Conflict("exists".to_string()).to_string(),
        "Bundle conflict: exists"
    );
    assert_eq!(
        AppError::BundleNotFound("missing".to_string()).to_string(),
        "Bundle not found: missing"
    );
    assert_eq!(
        AppError::BundleOutputNotReady("not ready".to_string()).to_string(),
        "Bundle output root is not ready: not ready"
    );
    assert_eq!(
        AppError::Quarantine("q".to_string()).to_string(),
        "Quarantine error: q"
    );
    assert_eq!(
        AppError::QuarantineConflict("qc".to_string()).to_string(),
        "Quarantine conflict: qc"
    );
    assert_eq!(
        AppError::QuarantineOutputNotReady("qr".to_string()).to_string(),
        "Quarantine output root is not ready: qr"
    );
    assert_eq!(
        AppError::Internal("boom".to_string()).to_string(),
        "Internal error: boom"
    );
}

#[test]
fn display_for_unit_variants_emits_a_human_readable_message() {
    assert_eq!(
        AppError::BundleOutputNotConfigured.to_string(),
        "Bundle output root is not configured"
    );
    assert_eq!(
        AppError::QuarantineOutputNotConfigured.to_string(),
        "Quarantine output path is not configured"
    );
}

#[test]
fn display_for_profile_load_uses_safe_message_rather_than_raw_detail() {
    let raw_detail = "this should not appear";
    let display = AppError::ProfileLoad(raw_detail.to_string()).to_string();
    assert!(
        display.contains(PROFILE_LOAD_SAFE_MESSAGE),
        "display should use safe message, got {display}"
    );
    assert!(
        !display.contains(raw_detail),
        "display must not leak inner detail, got {display}"
    );
}

#[test]
fn from_hl7v2_error_maps_to_parse_variant() {
    let app_error: AppError = hl7v2::Error::InvalidSegmentId.into();
    assert!(matches!(app_error, AppError::Parse(_)));
}

#[test]
fn from_profile_load_error_returns_safe_app_error_variant() {
    let app_error: AppError =
        hl7v2::conformance::profile::ProfileLoadError::YamlParse("bad yaml".to_string()).into();
    assert!(
        matches!(&app_error, AppError::ProfileLoad(m) if m == PROFILE_LOAD_SAFE_MESSAGE),
        "got {app_error:?}"
    );
}
