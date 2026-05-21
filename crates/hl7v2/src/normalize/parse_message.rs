use crate::model::{Error, Message};
use crate::parser::parse;

pub(super) fn parse_message(bytes: &[u8]) -> Result<Message, Error> {
    parse(bytes)
}
