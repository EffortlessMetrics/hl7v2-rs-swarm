use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::*;

/// Build a structured explanation report for a loaded profile.
#[must_use]
pub fn explain_profile(
    profile_name: impl Into<String>,
    profile_yaml: &str,
    profile: &Profile,
    lint_report: &ProfileLintReport,
) -> ProfileExplainReport {
    let required_fields = build_required_fields(profile);
    let table_code_counts = table_code_counts(profile);

    ProfileExplainReport {
        profile: profile_name.into(),
        profile_sha256: compute_profile_sha256(profile_yaml),
        message_structure: profile.message_structure.clone(),
        version: profile.version.clone(),
        message_type: profile.message_type.clone(),
        parent: profile.parent.clone(),
        summary: build_summary(profile, required_fields.len()),
        segments: build_segments(profile),
        required_fields,
        field_constraints: build_field_constraints(profile),
        length_rules: build_length_rules(profile),
        datatype_rules: build_datatype_rules(profile),
        value_sets: build_value_sets(profile, &table_code_counts),
        rules: build_rules(profile),
        hl7_tables: build_tables(profile),
        table_precedence: profile.table_precedence.clone(),
        expression_guardrails: build_expression_guardrails(profile),
        lint: build_lint_summary(lint_report),
    }
}

fn build_required_fields(profile: &Profile) -> Vec<ProfileExplainRequiredField> {
    profile
        .constraints
        .iter()
        .filter(|c| c.required)
        .map(|c| ProfileExplainRequiredField {
            path: c.path.clone(),
            conditional: c.when.is_some(),
        })
        .collect()
}
fn table_code_counts(profile: &Profile) -> HashMap<&str, usize> {
    profile
        .hl7_tables
        .iter()
        .map(|t| (t.id.as_str(), t.codes.len()))
        .collect()
}
fn build_summary(profile: &Profile, required_field_count: usize) -> ProfileExplainSummary {
    ProfileExplainSummary {
        segment_count: profile.segments.len(),
        required_field_count,
        field_constraint_count: profile.constraints.len(),
        length_rule_count: profile.lengths.len(),
        datatype_rule_count: profile.datatypes.len(),
        advanced_datatype_rule_count: profile.advanced_datatypes.len(),
        value_set_count: profile.valuesets.len(),
        cross_field_rule_count: profile.cross_field_rules.len(),
        temporal_rule_count: profile.temporal_rules.len(),
        contextual_rule_count: profile.contextual_rules.len(),
        custom_rule_count: profile.custom_rules.len(),
        hl7_table_count: profile.hl7_tables.len(),
    }
}
fn build_segments(profile: &Profile) -> Vec<ProfileExplainSegment> {
    profile
        .segments
        .iter()
        .map(|s| ProfileExplainSegment { id: s.id.clone() })
        .collect()
}

fn build_field_constraints(profile: &Profile) -> Vec<ProfileExplainConstraint> {
    profile
        .constraints
        .iter()
        .map(|constraint| {
            let (component_min, component_max) = constraint
                .components
                .as_ref()
                .map(|components| (components.min, components.max))
                .unwrap_or((None, None));
            let allowed_values = constraint.r#in.clone().unwrap_or_default();
            ProfileExplainConstraint {
                path: constraint.path.clone(),
                required: constraint.required,
                conditional: constraint.when.is_some(),
                component_min,
                component_max,
                allowed_value_count: allowed_values.len(),
                allowed_values,
                pattern: constraint.pattern.clone(),
            }
        })
        .collect()
}

fn build_length_rules(profile: &Profile) -> Vec<ProfileExplainLengthRule> {
    profile
        .lengths
        .iter()
        .map(|length| ProfileExplainLengthRule {
            path: length.path.clone(),
            max: length.max,
            policy: length.policy.clone(),
        })
        .collect()
}

fn build_datatype_rules(profile: &Profile) -> Vec<ProfileExplainDatatypeRule> {
    profile
        .datatypes
        .iter()
        .map(|datatype| ProfileExplainDatatypeRule {
            path: datatype.path.clone(),
            datatype: datatype.r#type.clone(),
            kind: "simple".to_string(),
            pattern: None,
            min_length: None,
            max_length: None,
            format: None,
            checksum: None,
        })
        .chain(
            profile
                .advanced_datatypes
                .iter()
                .map(|datatype| ProfileExplainDatatypeRule {
                    path: datatype.path.clone(),
                    datatype: datatype.r#type.clone(),
                    kind: "advanced".to_string(),
                    pattern: datatype.pattern.clone(),
                    min_length: datatype.min_length,
                    max_length: datatype.max_length,
                    format: datatype.format.clone(),
                    checksum: datatype.checksum.clone(),
                }),
        )
        .collect()
}

fn build_value_sets(
    profile: &Profile,
    table_code_counts: &HashMap<&str, usize>,
) -> Vec<ProfileExplainValueSet> {
    profile
        .valuesets
        .iter()
        .map(|valueset| {
            let table_code_count = table_code_counts
                .get(valueset.name.as_str())
                .copied()
                .unwrap_or(0);
            let source = if !valueset.codes.is_empty() {
                "inline"
            } else if table_code_count > 0 {
                "hl7_table"
            } else {
                "empty"
            };
            ProfileExplainValueSet {
                name: valueset.name.clone(),
                path: valueset.path.clone(),
                source: source.to_string(),
                inline_code_count: valueset.codes.len(),
                table_code_count,
            }
        })
        .collect()
}

fn build_rules(profile: &Profile) -> ProfileExplainRules {
    ProfileExplainRules {
        cross_field: profile
            .cross_field_rules
            .iter()
            .map(|rule| ProfileExplainRule {
                id: rule.id.clone(),
                description: rule.description.clone(),
            })
            .collect(),
        temporal: profile
            .temporal_rules
            .iter()
            .map(|rule| ProfileExplainRule {
                id: rule.id.clone(),
                description: rule.description.clone(),
            })
            .collect(),
        contextual: profile
            .contextual_rules
            .iter()
            .map(|rule| ProfileExplainRule {
                id: rule.id.clone(),
                description: rule.description.clone(),
            })
            .collect(),
        custom: profile
            .custom_rules
            .iter()
            .map(|rule| ProfileExplainRule {
                id: rule.id.clone(),
                description: rule.description.clone(),
            })
            .collect(),
    }
}

fn build_tables(profile: &Profile) -> Vec<ProfileExplainTable> {
    profile
        .hl7_tables
        .iter()
        .map(|table| ProfileExplainTable {
            id: table.id.clone(),
            name: table.name.clone(),
            version: table.version.clone(),
            code_count: table.codes.len(),
        })
        .collect()
}
fn build_expression_guardrails(profile: &Profile) -> ProfileExplainExpressionGuardrails {
    ProfileExplainExpressionGuardrails {
        max_depth: profile.expression_guardrails.max_depth,
        max_length: profile.expression_guardrails.max_length,
        allow_custom_scripts: profile.expression_guardrails.allow_custom_scripts,
    }
}
fn build_lint_summary(lint_report: &ProfileLintReport) -> ProfileExplainLintSummary {
    ProfileExplainLintSummary {
        valid: lint_report.valid,
        error_count: lint_report.error_count,
        warning_count: lint_report.warning_count,
        issue_count: lint_report.issue_count,
        ignored_or_unsupported: lint_report
            .issues
            .iter()
            .filter(|issue| profile_lint_issue_is_ignored_or_unsupported(issue))
            .cloned()
            .collect(),
    }
}
fn profile_lint_issue_is_ignored_or_unsupported(issue: &ProfileLintIssue) -> bool {
    issue.code.starts_with("unknown_")
        || issue.code.contains("unsupported")
        || issue.message.contains("ignored")
}
fn compute_profile_sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
