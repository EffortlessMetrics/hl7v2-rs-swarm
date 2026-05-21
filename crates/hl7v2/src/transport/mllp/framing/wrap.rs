use super::super::constants::{MLLP_END_1, MLLP_END_2, MLLP_START};

pub fn wrap_mllp(bytes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(bytes.len() + 3);
    buf.push(MLLP_START);
    buf.extend_from_slice(bytes);
    buf.push(MLLP_END_1);
    buf.push(MLLP_END_2);
    buf
}
