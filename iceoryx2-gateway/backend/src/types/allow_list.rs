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

//! A reusable allow list for scoping gateway mappings.

use alloc::vec::Vec;

/// An allow list defined using case-sensitive wildcard patterns.
///
/// `*` matches zero or more characters and `?` matches one character.
///
/// An empty list admits nothing. Use [`AllowList::all`] to admit every name.
#[derive(Debug, Default, Clone)]
pub struct AllowList {
    patterns: Vec<Vec<char>>,
}

impl AllowList {
    /// Builds an allow list from wildcard patterns.
    pub fn new<S: AsRef<str>>(patterns: &[S]) -> Self {
        Self {
            patterns: patterns
                .iter()
                .map(|pattern| pattern.as_ref().chars().collect())
                .collect(),
        }
    }

    /// Builds an allow list admitting every name.
    pub fn all() -> Self {
        Self::new(&["*"])
    }

    /// Whether the allowlist includes a pattern that permits the provided
    /// name.
    pub fn admits(&self, name: &str) -> bool {
        let name: Vec<char> = name.chars().collect();
        self.patterns.iter().any(|pattern| matches(pattern, &name))
    }
}

/// Whether the wildcard pattern matches the entire name.
///
/// The matcher is deliberately naive for simplicity and to avoid pulling in
/// additional dependencies. Matching cost grows with the number of
/// wildcards and the length of the name, which is accepted as names are
/// expected to be provided manually by users and short.
fn matches(pattern: &[char], name: &[char]) -> bool {
    match pattern {
        [] => name.is_empty(),
        ['*', rest @ ..] => (0..=name.len()).any(|skipped| matches(rest, &name[skipped..])),
        ['?', rest @ ..] => !name.is_empty() && matches(rest, &name[1..]),
        [literal, rest @ ..] => name.first() == Some(literal) && matches(rest, &name[1..]),
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
