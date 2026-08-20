// Copyright 2020 - developers of the `grammers` project.
// Copyright 2022 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` and `tdlib-rs` projects.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Errors that can occur during the parsing of [Type Language] definitions.
//!
//! [Type Language]: https://core.telegram.org/mtproto/TL

use std::fmt;

/// The error type for the parsing operation of [`Definition`]s.
///
/// [`Definition`]: tl/struct.Definition.html
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The definition is empty.
    Empty,

    /// One of the parameters from this definition was invalid.
    InvalidParam(ParamParseError),

    /// The name information is missing from the definition.
    MissingName,

    /// The type information is missing from the definition.
    MissingType,

    /// The parser does not know how to parse the definition.
    NotImplemented,

    /// The file contained an unknown separator (such as `---foo---`)
    UnknownSeparator,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty definition"),
            ParseError::InvalidParam(e) => write!(f, "invalid parameter: {}", e),
            ParseError::MissingName => write!(f, "missing name in definition"),
            ParseError::MissingType => write!(f, "missing type in definition"),
            ParseError::NotImplemented => write!(f, "definition parsing not implemented"),
            ParseError::UnknownSeparator => write!(f, "unknown separator in TL file"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::InvalidParam(e) => Some(e),
            _ => None,
        }
    }
}

/// The error type for the parsing operation of [`Parameter`]s.
///
/// [`Parameter`]: tl/struct.Parameter.html
#[derive(Debug, PartialEq, Eq)]
pub enum ParamParseError {
    /// The parameter was empty.
    Empty,

    /// The generic argument was invalid.
    InvalidGeneric,

    /// The parser does not know how to parse the parameter.
    NotImplemented,
}

impl fmt::Display for ParamParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamParseError::Empty => write!(f, "empty parameter"),
            ParamParseError::InvalidGeneric => write!(f, "invalid generic argument"),
            ParamParseError::NotImplemented => write!(f, "parameter parsing not implemented"),
        }
    }
}

impl std::error::Error for ParamParseError {}