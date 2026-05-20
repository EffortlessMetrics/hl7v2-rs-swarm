use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::AppError;
use crate::PROFILE_LOAD_SAFE_MESSAGE;
use crate::audit;
use crate::models::ErrorResponse;

struct ErrorSpec<'a> {
    status: StatusCode,
    code: &'a str,
    message: String,
    safe_detail: &'a str,
    location: Option<&'a str>,
    next_action: &'a str,
}

impl AppError {
    fn to_error_spec(&self) -> ErrorSpec<'_> {
        match self {
            AppError::Parse(msg) => ErrorSpec {
                status: StatusCode::BAD_REQUEST,
                code: "PARSE_ERROR",
                message: msg.clone(),
                safe_detail: "The request message could not be parsed as HL7 v2. Raw message content is not echoed.",
                location: Some("message"),
                next_action: "Check the MSH segment, segment terminators, encoding, and mllp_framed setting.",
            },
            AppError::ProfileLoad(_) => ErrorSpec {
                status: StatusCode::BAD_REQUEST,
                code: "PROFILE_LOAD_ERROR",
                message: PROFILE_LOAD_SAFE_MESSAGE.to_string(),
                safe_detail: "The supplied inline profile could not be loaded. Raw profile content is not echoed.",
                location: Some("profile"),
                next_action: "Run profile lint on the profile, then retry validation with the corrected profile.",
            },
            AppError::Validation(msg) => ErrorSpec {
                status: StatusCode::BAD_REQUEST,
                code: "VALIDATION_ERROR",
                message: msg.clone(),
                safe_detail: "The request failed validation before a successful evidence response was produced.",
                location: None,
                next_action: "Check request parameters, schema-version fields, and validation issue paths where available.",
            },
            AppError::Redaction(msg) => ErrorSpec {
                status: StatusCode::BAD_REQUEST,
                code: "REDACTION_ERROR",
                message: msg.clone(),
                safe_detail: "The redaction policy or redaction run failed before a safe response was produced.",
                location: Some("redaction_policy"),
                next_action: "Check safe-analysis policy paths, actions, reasons, and required-field matches before retrying.",
            },
            AppError::BundleOutputNotConfigured => ErrorSpec {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "BUNDLE_OUTPUT_NOT_CONFIGURED",
                message: "server bundle output root is not configured".to_string(),
                safe_detail: "The server cannot create evidence bundles until an operator configures a bundle root.",
                location: Some("bundle_output_root"),
                next_action: "Configure the server bundle output root and verify readiness before retrying.",
            },
            AppError::BundleOutputNotReady(msg) => ErrorSpec {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "BUNDLE_OUTPUT_NOT_READY",
                message: msg.clone(),
                safe_detail: "The configured bundle output root is not currently writable or available.",
                location: Some("bundle_output_root"),
                next_action: "Check server filesystem permissions and readiness before retrying.",
            },
            AppError::Bundle(msg) => ErrorSpec {
                status: StatusCode::BAD_REQUEST,
                code: "BUNDLE_ERROR",
                message: msg.clone(),
                safe_detail: "The bundle request could not be accepted. Server responses use safe bundle identifiers.",
                location: Some("bundle_id"),
                next_action: "Use a simple bundle id without path traversal and retry after validating inputs.",
            },
            AppError::Conflict(msg) => ErrorSpec {
                status: StatusCode::CONFLICT,
                code: "BUNDLE_EXISTS",
                message: msg.clone(),
                safe_detail: "The requested bundle output already exists under the configured root.",
                location: Some("bundle_id"),
                next_action: "Choose a new bundle id or replay the existing bundle instead of overwriting it.",
            },
            AppError::BundleNotFound(msg) => ErrorSpec {
                status: StatusCode::NOT_FOUND,
                code: "BUNDLE_NOT_FOUND",
                message: msg.clone(),
                safe_detail: "The requested bundle id was not found under the configured root.",
                location: Some("bundle_id"),
                next_action: "Check the bundle id from the bundle creation receipt and retry.",
            },
            AppError::QuarantineOutputNotConfigured => ErrorSpec {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "QUARANTINE_OUTPUT_NOT_CONFIGURED",
                message: "server quarantine output is enabled but no path is configured"
                    .to_string(),
                safe_detail: "The server cannot write quarantine artifacts until an operator configures a quarantine root.",
                location: Some("quarantine.path"),
                next_action: "Configure the quarantine output path or disable quarantine output before retrying.",
            },
            AppError::QuarantineOutputNotReady(msg) => ErrorSpec {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "QUARANTINE_OUTPUT_NOT_READY",
                message: msg.clone(),
                safe_detail: "The configured quarantine output root is not currently writable or available.",
                location: Some("quarantine.path"),
                next_action: "Check server filesystem permissions and readiness before retrying.",
            },
            AppError::Quarantine(msg) => ErrorSpec {
                status: StatusCode::BAD_REQUEST,
                code: "QUARANTINE_ERROR",
                message: msg.clone(),
                safe_detail: "The quarantine request could not be written as configured.",
                location: Some("quarantine"),
                next_action: "Check quarantine artifact settings and retry with reviewed redaction inputs.",
            },
            AppError::QuarantineConflict(msg) => ErrorSpec {
                status: StatusCode::CONFLICT,
                code: "QUARANTINE_EXISTS",
                message: msg.clone(),
                safe_detail: "The generated quarantine output collided with existing output.",
                location: Some("quarantine"),
                next_action: "Retry the request or inspect the existing quarantine output before sharing evidence.",
            },
            AppError::Internal(msg) => ErrorSpec {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "INTERNAL_ERROR",
                message: msg.clone(),
                safe_detail: "The server hit an internal failure. Raw request payloads are not included in this response.",
                location: None,
                next_action: "Check server logs and readiness, then retry with the same request only if disclosure policy allows.",
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let spec = self.to_error_spec();

        tracing::warn!(
            target: "hl7v2_server::evidence",
            event = audit::EVENT_ERROR,
            status = spec.status.as_u16(),
            error_code = spec.code,
            "request failed"
        );

        let mut error = ErrorResponse::new(spec.code, spec.message)
            .with_safe_detail(spec.safe_detail)
            .with_suggested_next_action(spec.next_action);

        if let Some(location) = spec.location {
            error = error.with_location(location);
        }

        (spec.status, Json(error)).into_response()
    }
}
