//! Email validation helpers used by HL7 datatype validation.

/// Validates a basic email structure.
///
/// This intentionally performs minimal checks used by HL7 `XTN`-style
/// validation paths: one `@`, non-empty local/domain parts, and at least one
/// dot in the domain section.
pub fn is_basic_email(value: &str) -> bool {
    let Some((local_part, domain_part)) = split_email_parts(value) else {
        return false;
    };

    has_non_empty_parts(local_part, domain_part) && domain_has_dot(domain_part)
}

fn split_email_parts(value: &str) -> Option<(&str, &str)> {
    let (local_part, domain_part) = value.split_once('@')?;
    if domain_part.contains('@') {
        return None;
    }
    Some((local_part, domain_part))
}

fn has_non_empty_parts(local_part: &str, domain_part: &str) -> bool {
    !local_part.is_empty() && !domain_part.is_empty()
}

fn domain_has_dot(domain_part: &str) -> bool {
    domain_part.contains('.')
}

#[cfg(test)]
mod tests {
    use super::is_basic_email;

    #[test]
    fn accepts_basic_email() {
        assert!(is_basic_email("user@example.com"));
    }

    #[test]
    fn rejects_invalid_shapes() {
        assert!(!is_basic_email("noatsign"));
        assert!(!is_basic_email("a@@b.com"));
        assert!(!is_basic_email("@example.com"));
        assert!(!is_basic_email("user@"));
        assert!(!is_basic_email("user@nodot"));
    }
}
