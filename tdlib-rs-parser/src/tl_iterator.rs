// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` and `tdlib-rs` projects.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::errors::ParseError;
use crate::tl::{Category, Definition};

const DEFINITION_SEP: char = ';';
const FUNCTIONS_SEP: &str = "---functions---";
const TYPES_SEP: &str = "---types---";

/// An iterator over [Type Language] definitions.
///
/// [Type Language]: https://core.telegram.org/mtproto/TL
pub struct TlIterator {
    contents: String,
    index: usize,
    category: Category,
}

impl TlIterator {
    pub(crate) fn new(contents: String) -> Self {
        TlIterator {
            contents,
            index: 0,
            category: Category::Types,
        }
    }
}

impl Iterator for TlIterator {
    type Item = Result<Definition, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.contents.len() {
                return None;
            }

            // Find the next definition separator (`;`), respecting quoted strings
            // and skipping comments (`// ...`).
            let (end, is_empty) = self.find_definition_end()?;

            let definition = self.contents[self.index..end].trim();
            self.index = end + DEFINITION_SEP.len_utf8();

            if !is_empty {
                // Get rid of the leading separator and adjust category
                let definition = if definition.starts_with("---") {
                    if let Some(rest) = definition.strip_prefix(FUNCTIONS_SEP) {
                        self.category = Category::Functions;
                        rest.trim()
                    } else if let Some(rest) = definition.strip_prefix(TYPES_SEP) {
                        self.category = Category::Types;
                        rest.trim()
                    } else {
                        return Some(Err(ParseError::UnknownSeparator));
                    }
                } else {
                    definition
                };

                // Yield the fixed definition
                return Some(match definition.parse::<Definition>() {
                    Ok(mut d) => {
                        d.category = self.category;
                        Ok(d)
                    }
                    x => x,
                });
            }
        }
    }
}

impl TlIterator {
    /// Scans from `self.index` to find the next ';' that is not inside a comment
    /// or a quoted string. Returns `(end_index, is_empty)` where `end_index`
    /// points to the position of the ';' (or the end of the string) and `is_empty`
    /// indicates whether the definition was empty (only whitespace/comments).
    fn find_definition_end(&self) -> Option<(usize, bool)> {
        let chars = self.contents[self.index..].char_indices();
        let mut in_comment = false;
        let mut in_quotes = false;
        let mut is_empty = true;

        for (idx, c) in chars {
            if !in_comment {
                if !in_quotes && c == '/' {
                    // Check if the next char is also '/'
                    let next_idx = self.index + idx + c.len_utf8();
                    if next_idx < self.contents.len() {
                        let next_char = self.contents[next_idx..].chars().next().unwrap_or('\0');
                        if next_char == '/' {
                            in_comment = true;
                            continue;
                        }
                    }
                }

                if !in_comment && !in_quotes && c == '"' {
                    in_quotes = true;
                } else if in_quotes && c == '"' && !self.escaped_char(idx) {
                    in_quotes = false;
                }

                if !in_comment && !c.is_whitespace() {
                    is_empty = false;
                }

                if !in_comment && !in_quotes && c == DEFINITION_SEP {
                    return Some((self.index + idx, is_empty));
                }
            } else if c == '\n' {
                in_comment = false;
            }
        }

        // No separator found, use the end of string
        Some((self.contents.len(), is_empty))
    }

    /// Checks if the character at the given offset is escaped by a backslash.
    #[inline]
    fn escaped_char(&self, offset: usize) -> bool {
        let idx = self.index + offset;
        idx > self.index && self.contents[idx - 1..].chars().next() == Some('\\')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ParseError;

    #[test]
    fn parse_bad_separator() {
        let mut it = TlIterator::new("---foo---".into());
        assert_eq!(it.next(), Some(Err(ParseError::UnknownSeparator)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn parse_file() {
        let mut it = TlIterator::new(
            "
            // leading; comment
            first = t; // inline comment
            second and bad;
            third = t;
            // trailing comment
        "
            .into(),
        );

        assert_eq!(it.next().unwrap().unwrap().name, "first");
        assert!(it.next().unwrap().is_err());
        assert_eq!(it.next().unwrap().unwrap().name, "third");
        assert_eq!(it.next(), None);
    }

    #[test]
    fn quoted_semicolon() {
        let mut it = TlIterator::new(
            r#"quoted = "hello; world"; // not separator"#.into()
        );
        let def = it.next().unwrap().unwrap();
        assert_eq!(def.name, "quoted");
        assert_eq!(it.next(), None);
    }
}