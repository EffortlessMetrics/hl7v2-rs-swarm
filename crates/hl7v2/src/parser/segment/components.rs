use crate::escape::unescape_text;
use crate::model::{Atom, Comp, Delims, Error, Rep};

pub(super) fn parse_rep(rep_str: &str, delims: &Delims) -> Result<Rep, Error> {
    if rep_str == "\"\"" {
        return Ok(Rep {
            comps: vec![Comp {
                subs: vec![Atom::Null],
            }],
        });
    }

    if rep_str.contains('\n') || rep_str.contains('\r') {
        return Err(Error::InvalidRepFormat {
            details: "Repetition contains invalid line break characters".to_string(),
        });
    }

    let comp_count = rep_str.matches(delims.comp).count() + 1;
    let mut comps = Vec::with_capacity(comp_count);

    for (i, comp_str) in rep_str.split(delims.comp).enumerate() {
        let comp = parse_comp(comp_str, delims).map_err(|e| match e {
            Error::InvalidCompFormat { .. } => e,
            _ => Error::InvalidCompFormat {
                details: format!("Component {}: {}", i, e),
            },
        })?;
        comps.push(comp);
    }

    Ok(Rep { comps })
}

fn parse_comp(comp_str: &str, delims: &Delims) -> Result<Comp, Error> {
    if comp_str.contains('\n') || comp_str.contains('\r') {
        return Err(Error::InvalidCompFormat {
            details: "Component contains invalid line break characters".to_string(),
        });
    }

    let sub_count = comp_str.matches(delims.sub).count() + 1;
    let mut subs = Vec::with_capacity(sub_count);

    for (i, sub_str) in comp_str.split(delims.sub).enumerate() {
        let atom = parse_atom(sub_str, delims).map_err(|e| match e {
            Error::InvalidSubcompFormat { .. } => e,
            _ => Error::InvalidSubcompFormat {
                details: format!("Subcomponent {}: {}", i, e),
            },
        })?;
        subs.push(atom);
    }

    Ok(Comp { subs })
}

fn parse_atom(atom_str: &str, delims: &Delims) -> Result<Atom, Error> {
    if atom_str == "\"\"" {
        return Ok(Atom::Null);
    }

    if atom_str.contains('\n') || atom_str.contains('\r') {
        return Err(Error::InvalidSubcompFormat {
            details: "Subcomponent contains invalid line break characters".to_string(),
        });
    }

    let unescaped = unescape_text(atom_str, delims)?;
    Ok(Atom::Text(unescaped))
}
