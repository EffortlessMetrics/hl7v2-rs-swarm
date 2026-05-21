use crate::model::Error;

use super::constants::MLLP_START;
use super::errors::MllpError;

mod detect;
mod parse;
mod wrap;

pub use detect::{find_complete_mllp_message, is_mllp_framed};
pub use wrap::wrap_mllp;

pub fn unwrap_mllp(bytes: &[u8]) -> Result<&[u8], Error> {
    validate_start_block(bytes).map_err(|err| Error::Framing(err.to_string()))?;

    let end_pos = parse::find_mllp_end(bytes)?;
    Ok(&bytes[1..end_pos])
}

pub fn unwrap_mllp_checked(bytes: &[u8]) -> Result<&[u8], MllpError> {
    validate_start_block(bytes)?;

    let end_pos = parse::find_mllp_end_checked(bytes)?;
    Ok(&bytes[1..end_pos])
}

pub fn unwrap_mllp_owned(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    unwrap_mllp(bytes).map(<[u8]>::to_vec)
}

pub fn unwrap_mllp_owned_checked(bytes: &[u8]) -> Result<Vec<u8>, MllpError> {
    unwrap_mllp_checked(bytes).map(<[u8]>::to_vec)
}

fn validate_start_block(bytes: &[u8]) -> Result<(), MllpError> {
    if bytes.is_empty() || bytes[0] != MLLP_START {
        return Err(MllpError::MissingStartBlock);
    }

    Ok(())
}
