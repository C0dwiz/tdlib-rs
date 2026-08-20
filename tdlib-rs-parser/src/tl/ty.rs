// Copyright 2020 - developers of the `grammers` project.
// Copyright 2022 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` and `tdlib-rs` projects.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::fmt;
use std::str::FromStr;

use crate::errors::ParamParseError;

/// The type of a definition or a parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    /// The name of the type.
    pub name: String,

    /// Whether this type is bare or boxed.
    /// A type is "bare" if its name starts with a lowercase letter,
    /// "boxed" if it starts with an uppercase letter.
    pub bare: bool,

    /// If the type has a generic argument, which is its type.
    pub generic_arg: Option<Box<Type>>,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(generic_arg) = &self.generic_arg {
            write!(f, "<{}>", generic_arg)?;
        }
        Ok(())
    }
}

impl FromStr for Type {
    type Err = ParamParseError;

    /// Parses a type.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdlib_rs_parser::tl::Type;
    ///
    /// assert!("vector<int>".parse::<Type>().is_ok());
    /// assert!("vector < int >".parse::<Type>().is_ok());
    /// ```
    fn from_str(ty: &str) -> Result<Self, Self::Err> {
        let ty = ty.trim();
        if ty.is_empty() {
            return Err(ParamParseError::Empty);
        }

        // Parse `type<generic_arg>` with support for optional spaces.
        // Find the matching closing '>' by counting angle brackets.
        let (name_part, generic_arg) = if let Some(open_pos) = ty.find('<') {
            // Find the matching closing '>'
            let mut depth = 0;
            let mut close_pos = None;
            for (i, ch) in ty[open_pos..].char_indices() {
                match ch {
                    '<' => depth += 1,
                    '>' => {
                        depth -= 1;
                        if depth == 0 {
                            close_pos = Some(open_pos + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            match close_pos {
                Some(pos) => {
                    let name = ty[..open_pos].trim();
                    if name.is_empty() {
                        return Err(ParamParseError::Empty);
                    }
                    let arg_str = ty[open_pos + 1..pos].trim();
                    if arg_str.is_empty() {
                        return Err(ParamParseError::InvalidGeneric);
                    }
                    let arg = Type::from_str(arg_str)?;
                    (name, Some(Box::new(arg)))
                }
                None => return Err(ParamParseError::InvalidGeneric),
            }
        } else {
            (ty, None)
        };

        // Determine bareness based on the first character of the name
        let bare = name_part
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false);

        Ok(Self {
            name: name_part.to_string(),
            bare,
            generic_arg,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_empty() {
        assert_eq!(Type::from_str(""), Err(ParamParseError::Empty));
        assert_eq!(Type::from_str("   "), Err(ParamParseError::Empty));
    }

    #[test]
    fn check_simple() {
        assert_eq!(
            Type::from_str("foo"),
            Ok(Type {
                name: "foo".into(),
                bare: true,
                generic_arg: None,
            })
        );
        assert_eq!(
            Type::from_str("  foo  "),
            Ok(Type {
                name: "foo".into(),
                bare: true,
                generic_arg: None,
            })
        );
    }

    #[test]
    fn check_bare() {
        assert!(matches!(Type::from_str("foo"), Ok(Type { bare: true, .. })));
        assert!(matches!(
            Type::from_str("Foo"),
            Ok(Type { bare: false, .. })
        ));
        assert!(matches!(
            Type::from_str("  Foo  "),
            Ok(Type { bare: false, .. })
        ));
    }

    #[test]
    fn check_generic_arg() {
        assert!(matches!(
            Type::from_str("foo"),
            Ok(Type {
                generic_arg: None,
                ..
            })
        ));
        assert!(match Type::from_str("foo<bar>") {
            Ok(Type {
                generic_arg: Some(x),
                ..
            }) => *x == "bar".parse().unwrap(),
            _ => false,
        });
        assert!(match Type::from_str("foo<bar<baz>>") {
            Ok(Type {
                generic_arg: Some(x),
                ..
            }) => *x == "bar<baz>".parse().unwrap(),
            _ => false,
        });
        assert!(match Type::from_str("foo < bar >") {
            Ok(Type {
                generic_arg: Some(x),
                ..
            }) => *x == "bar".parse().unwrap(),
            _ => false,
        });
        assert!(match Type::from_str("foo < bar < baz > >") {
            Ok(Type {
                generic_arg: Some(x),
                ..
            }) => *x == "bar < baz >".parse().unwrap(),
            _ => false,
        });
    }

    #[test]
    fn check_invalid_generic() {
        assert_eq!(
            Type::from_str("foo<bar"),
            Err(ParamParseError::InvalidGeneric)
        );
        assert_eq!(
            Type::from_str("foo< >"),
            Err(ParamParseError::InvalidGeneric)
        );
        assert_eq!(
            Type::from_str("<>"),
            Err(ParamParseError::Empty)
        );
    }
}