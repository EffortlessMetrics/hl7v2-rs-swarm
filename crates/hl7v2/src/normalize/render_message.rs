use crate::model::Message;
use crate::writer::write;

pub(super) fn render_message(message: &Message) -> Vec<u8> {
    write(message)
}
