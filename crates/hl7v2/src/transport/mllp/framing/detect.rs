use super::super::constants::MLLP_START;
use super::parse::find_mllp_end_index;

pub fn is_mllp_framed(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes[0] == MLLP_START
}

pub fn find_complete_mllp_message(bytes: &[u8]) -> Option<usize> {
    if !is_mllp_framed(bytes) {
        return None;
    }

    find_mllp_end_index(bytes).map(|end_pos| end_pos + 2)
}
