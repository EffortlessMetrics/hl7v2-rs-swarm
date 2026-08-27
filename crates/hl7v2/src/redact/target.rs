use crate::model::{Atom, Comp, Field, Rep};

use super::digest::compute_sha256;
use super::path::ParsedRedactionPath;
use super::text::{atom_to_text, comp_to_text, field_to_text, rep_to_text};
use super::types::RedactionAction;

pub(crate) fn apply_redaction_target(
    field: &mut Field,
    path: &ParsedRedactionPath,
    action: RedactionAction,
    delims: &crate::Delims,
) -> bool {
    for_each_target(field, path, &mut |target| match action {
        RedactionAction::Hash => target.hash(delims),
        RedactionAction::Drop => target.drop_value(),
        RedactionAction::Retain => {}
    })
}

pub(crate) fn replace_redaction_target(
    field: &mut Field,
    path: &ParsedRedactionPath,
    replacement: &str,
) -> bool {
    for_each_target(field, path, &mut |target| {
        target.replace_with_text(replacement.to_string());
    })
}

enum RedactionTarget<'a> {
    Field(&'a mut Field),
    Rep(&'a mut Rep),
    Comp(&'a mut Comp),
    Atom(&'a mut Atom),
}

impl RedactionTarget<'_> {
    fn hash(self, delims: &crate::Delims) {
        let value = match &self {
            Self::Field(field) => field_to_text(field, delims),
            Self::Rep(rep) => rep_to_text(rep, delims),
            Self::Comp(comp) => comp_to_text(comp, delims),
            Self::Atom(atom) => atom_to_text(atom).to_string(),
        };
        self.replace_with_text(format!("hash:sha256:{}", compute_sha256(&value)));
    }

    fn drop_value(self) {
        self.replace_with_text(String::new());
    }

    fn replace_with_text(self, replacement: String) {
        match self {
            Self::Field(field) => {
                *field = Field::from_text(replacement);
            }
            Self::Rep(rep) => {
                *rep = Rep::from_text(replacement);
            }
            Self::Comp(comp) => {
                *comp = Comp::from_text(replacement);
            }
            Self::Atom(atom) => {
                *atom = Atom::Text(replacement);
            }
        }
    }
}

/// Visit every value a redaction path selects, returning whether any existed.
///
/// A path that names no field repetition selects its target in *every*
/// repetition of the field, mirroring how a path that names no segment
/// repetition applies to every matching segment. Redacting only the first
/// repetition would leave the remaining ones in the output: `PID.13.1` over
/// `555-1111^HOME~555-2222^WORK` must not keep `555-2222`. Callers that mean a
/// single repetition say so with the explicit selector, as in `PID.13[2].1`.
fn for_each_target(
    field: &mut Field,
    path: &ParsedRedactionPath,
    visit: &mut dyn FnMut(RedactionTarget<'_>),
) -> bool {
    if path.field_repetition.is_none() && path.component.is_none() {
        visit(RedactionTarget::Field(field));
        return true;
    }

    match path.field_repetition {
        Some(repetition) => {
            let Some(index) = repetition.checked_sub(1) else {
                return false;
            };
            let Some(rep) = field.reps.get_mut(index) else {
                return false;
            };
            visit_rep(rep, path, visit)
        }
        None => {
            let mut visited = false;
            for rep in &mut field.reps {
                visited |= visit_rep(rep, path, visit);
            }
            visited
        }
    }
}

fn visit_rep(
    rep: &mut Rep,
    path: &ParsedRedactionPath,
    visit: &mut dyn FnMut(RedactionTarget<'_>),
) -> bool {
    let Some(component) = path.component else {
        visit(RedactionTarget::Rep(rep));
        return true;
    };

    let Some(component_index) = component.checked_sub(1) else {
        return false;
    };
    let Some(comp) = rep.comps.get_mut(component_index) else {
        return false;
    };
    let Some(subcomponent) = path.subcomponent else {
        visit(RedactionTarget::Comp(comp));
        return true;
    };

    let Some(subcomponent_index) = subcomponent.checked_sub(1) else {
        return false;
    };
    match comp.subs.get_mut(subcomponent_index) {
        Some(atom) => {
            visit(RedactionTarget::Atom(atom));
            true
        }
        None => false,
    }
}
