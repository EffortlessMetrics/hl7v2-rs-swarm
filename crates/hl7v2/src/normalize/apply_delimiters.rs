use crate::model::{Delims, Message};

pub(super) fn apply_delimiters(message: &mut Message, canonical_delims: bool) {
    if canonical_delims {
        message.delims = Delims::default();
    }
}
