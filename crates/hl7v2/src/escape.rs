//! HL7 v2 escape sequence handling.
//!
//! This module provides functions for escaping and unescaping HL7 v2 text
//! according to the standard escape sequences defined in the HL7 v2 specification.
//!
//! # Escape Sequences
//!
//! HL7 v2 uses escape sequences to represent delimiter characters within field values:
//! - `\F\` - Field separator
//! - `\S\` - Component separator
//! - `\R\` - Repetition separator
//! - `\E\` - Escape character
//! - `\T\` - Subcomponent separator
//!
//! # Example
//!
//! ```
//! use hl7v2::{Delims, escape_text, unescape_text};
//!
//! let delims = Delims::default();
//! let text = "test|value";
//! let escaped = escape_text(text, &delims);
//! assert_eq!(escaped, "test\\F\\value");
//!
//! let unescaped = unescape_text(&escaped, &delims).unwrap();
//! assert_eq!(unescaped, text);
//! ```

#![expect(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::manual_let_else,
    clippy::missing_errors_doc,
    clippy::string_slice,
    reason = "pre-existing escape implementation debt moved from staged microcrate into hl7v2; cleanup is split from topology collapse"
)]

mod decode;
mod detect;
mod encode;

pub use decode::unescape_text;
pub use detect::{needs_escaping, needs_unescaping};
pub use encode::escape_text;

#[cfg(test)]
mod tests;
