use crate::model::Error;

use super::super::constants::{MLLP_END_1, MLLP_END_2};
use super::super::errors::MllpError;

pub fn find_mllp_end(bytes: &[u8]) -> Result<usize, Error> {
    find_mllp_end_index(bytes).ok_or_else(|| {
        Error::Framing("Missing MLLP end block sequence (0x1C 0x0D)".to_string())
    })
}

pub fn find_mllp_end_checked(bytes: &[u8]) -> Result<usize, MllpError> {
    find_mllp_end_index(bytes).ok_or(MllpError::MissingEndBlock)
}

pub fn find_mllp_end_index(bytes: &[u8]) -> Option<usize> {
    for i in 0..bytes.len().saturating_sub(1) {
        if bytes[i] == MLLP_END_1 && bytes[i + 1] == MLLP_END_2 {
            return Some(i);
        }
    }

    None
}
