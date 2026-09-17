// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use alloc::string::String;
use alloc::vec::Vec;

/// An allow list defined using case-sensitive wildcard patterns.
///
/// `*` matches zero or more characters and `?` matches one character.
///
/// An empty list admits nothing. Use [`AllowList::all`] to admit every name.
#[derive(Debug, Default, Clone)]
pub struct AllowList {
    patterns: Vec<String>,
}

impl AllowList {
    /// Builds an allow list from wildcard patterns.
    pub fn new<S: AsRef<str>>(patterns: &[S]) -> Self {
        Self {
            patterns: patterns.iter().map(|p| String::from(p.as_ref())).collect(),
        }
    }

    /// Builds an allow list admitting every name.
    pub fn all() -> Self {
        Self::new(&["*"])
    }

    /// Whether the allowlist includes a pattern that permits the provided
    /// name.
    pub fn admits(&self, name: &str) -> bool {
        self.patterns.iter().any(|pattern| matches(pattern, name))
    }
}

/// The most recent `*`, the pattern after it and the name that pattern was
/// last compared against.
struct Backtrack<'a> {
    pattern: &'a str,
    name: &'a str,
}

/// Whether the pattern, which may contain `*` and `?`, matches the whole
/// name.
///
/// Pattern and name are compared from the left. A literal matches only
/// itself and a `?` matches any one character, each consuming one character
/// of the name.
///
/// ```text
/// cmd    against  cmd,    every character matches itself
/// cmd_?  against  cmd_1,  `?` matches 1
/// ```
///
/// A `*` stands for any run of characters, so matching searches for the runs
/// that make the pattern fit the name. A `*` begins representing nothing and
/// grows by one character whenever the pattern after it fails to match.
/// The search succeeds once the whole name is matched, and fails when no
/// `*` can represent the characters that do not match.
///
/// ```text
/// robot*/cmd  matched against  robot1/x/cmd
///
/// robot  matches literally, then `*` stands for nothing
/// /cmd   against 1/x/cmd, / cannot match 1, so `*` grows to include 1
/// /cmd   against /x/cmd, / matches, c cannot match x, so `*` grows to 1/
/// /cmd   against x/cmd, / cannot match x, so `*` grows to 1/x
/// /cmd   against /cmd, every character matches, so the whole name matches
/// ```
fn matches(pattern: &str, name: &str) -> bool {
    // Both hold what is still unmatched, shrinking from the left.
    let mut pattern = pattern;
    let mut name = name;
    let mut backtrack: Option<Backtrack> = None;

    while !name.is_empty() {
        match first(pattern) {
            // A `*` matches nothing for now, it grows only on a mismatch.
            Some('*') => {
                pattern = rest(pattern);
                backtrack = Some(Backtrack { pattern, name });
            }
            // A `?` matches whatever one character stands here.
            Some('?') => {
                pattern = rest(pattern);
                name = rest(name);
            }
            // Any other pattern character matches only itself.
            Some(literal) if Some(literal) == first(name) => {
                pattern = rest(pattern);
                name = rest(name);
            }
            // No match here, so grow the last `*` by one character and retry.
            _ => match backtrack {
                Some(last_star) => {
                    pattern = last_star.pattern;
                    name = rest(last_star.name);
                    backtrack = Some(Backtrack { pattern, name });
                }
                // Without a `*` to grow, the pattern cannot match.
                None => return false,
            },
        }
    }

    // The name is consumed, so only stars may be left over.
    pattern.chars().all(|c| c == '*')
}

fn first(text: &str) -> Option<char> {
    text.chars().next()
}

/// The text behind its first character.
fn rest(text: &str) -> &str {
    match first(text) {
        Some(character) => &text[character.len_utf8()..],
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn literal_pattern_admits_only_its_name() {
        let sut = AllowList::new(&["service"]);

        assert_that!(sut.admits("service"), eq true);
        assert_that!(sut.admits("service/child"), eq false);
        assert_that!(sut.admits("other"), eq false);
        assert_that!(sut.admits("parent/service"), eq false);
        assert_that!(sut.admits("parent/service/child"), eq false);
    }

    #[test]
    fn wildcard_matches_any_number_of_characters_including_slashes() {
        let sut = AllowList::new(&["/camera/*"]);

        assert_that!(sut.admits("/camera/front"), eq true);
        assert_that!(sut.admits("/camera/rear/depth"), eq true);
        assert_that!(sut.admits("/camera/"), eq true);
        assert_that!(sut.admits("secondary/camera/"), eq false);
        assert_that!(sut.admits("secondary/camera/front"), eq false);
        assert_that!(sut.admits("/lidar/front"), eq false);
    }

    #[test]
    fn wildcard_can_match_in_any_pattern_position() {
        let sut = AllowList::new(&["robot*/cmd_*"]);

        assert_that!(sut.admits("robot/cmd_vel"), eq true);
        assert_that!(sut.admits("robot42/cmd_speed/limit"), eq true);
        assert_that!(sut.admits("other42/cmd_vel"), eq false);
    }

    #[test]
    fn question_mark_matches_exactly_one_character() {
        let sut = AllowList::new(&["robot?/cmd"]);

        assert_that!(sut.admits("robot1/cmd"), eq true);
        assert_that!(sut.admits("robot//cmd"), eq true);
        assert_that!(sut.admits("robot/cmd"), eq false);
        assert_that!(sut.admits("robot12/cmd"), eq false);
    }

    #[test]
    fn trailing_question_mark_requires_one_more_character() {
        let sut = AllowList::new(&["robot?"]);

        assert_that!(sut.admits("robot1"), eq true);
        assert_that!(sut.admits("robots"), eq true);
        assert_that!(sut.admits("robot"), eq false);
        assert_that!(sut.admits("robot12"), eq false);
    }

    #[test]
    fn leading_question_mark_requires_one_preceding_character() {
        let sut = AllowList::new(&["?robot"]);

        assert_that!(sut.admits("1robot"), eq true);
        assert_that!(sut.admits("/robot"), eq true);
        assert_that!(sut.admits("robot"), eq false);
        assert_that!(sut.admits("12robot"), eq false);
    }

    #[test]
    fn adjacent_wildcards_are_equivalent_to_one() {
        let sut = AllowList::new(&["/camera/**"]);

        assert_that!(sut.admits("/camera/front/depth"), eq true);
    }

    #[test]
    fn all_admits_every_name() {
        let sut = AllowList::all();

        assert_that!(sut.admits("service"), eq true);
        assert_that!(sut.admits("/camera/front"), eq true);
    }

    #[test]
    fn entries_accumulate() {
        let sut = AllowList::new(&["service", "/camera/*"]);

        assert_that!(sut.admits("service"), eq true);
        assert_that!(sut.admits("/camera/front"), eq true);
        assert_that!(sut.admits("other"), eq false);
    }

    #[test]
    fn empty_list_admits_nothing() {
        assert_that!(AllowList::default().admits("service"), eq false);
    }
}
