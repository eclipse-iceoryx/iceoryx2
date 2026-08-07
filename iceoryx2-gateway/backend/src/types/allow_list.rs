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

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Trailing marker of a pattern admitting every name under a prefix.
const WILDCARD: &str = "**";

#[derive(Debug, Clone, Eq, PartialEq)]
enum Entry {
    Exact(String),
    Prefixed(String),
}

impl Entry {
    fn parse(pattern: &str) -> Self {
        match pattern.strip_suffix(WILDCARD) {
            Some(prefix) => Self::Prefixed(prefix.to_string()),
            None => Self::Exact(pattern.to_string()),
        }
    }

    fn admits(&self, name: &str) -> bool {
        match self {
            Self::Exact(entry) => entry == name,
            Self::Prefixed(prefix) => name.starts_with(prefix),
        }
    }
}

/// Names admitted by a mapping, written as exact names or `<prefix>**`
/// patterns.
///
/// An empty list admits nothing. Use [`AllowList::all`] to admit every name.
#[derive(Debug, Default, Clone)]
pub struct AllowList {
    entries: Vec<Entry>,
}

impl AllowList {
    /// Builds an allow list from exact names and prefix patterns.
    pub fn new<S: AsRef<str>>(patterns: &[S]) -> Self {
        Self {
            entries: patterns
                .iter()
                .map(|pattern| Entry::parse(pattern.as_ref()))
                .collect(),
        }
    }

    /// Builds an allow list admitting every name.
    pub fn all() -> Self {
        Self::new(&[WILDCARD])
    }

    /// Whether any entry admits `name`.
    pub fn admits(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.admits(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_entry_admits_only_its_name() {
        let sut = AllowList::new(&["service"]);

        assert!(sut.admits("service"));
        assert!(!sut.admits("service/child"));
        assert!(!sut.admits("other"));
    }

    #[test]
    fn prefixed_entry_admits_every_name_below_it() {
        let sut = AllowList::new(&["/camera/**"]);

        assert!(sut.admits("/camera/front"));
        assert!(sut.admits("/camera/rear/depth"));
        assert!(!sut.admits("/lidar/front"));
    }

    #[test]
    fn all_admits_every_name() {
        let sut = AllowList::all();

        assert!(sut.admits("service"));
        assert!(sut.admits("/camera/front"));
    }

    #[test]
    fn entries_accumulate() {
        let sut = AllowList::new(&["service", "/camera/**"]);

        assert!(sut.admits("service"));
        assert!(sut.admits("/camera/front"));
        assert!(!sut.admits("other"));
    }

    #[test]
    fn empty_list_admits_nothing() {
        assert!(!AllowList::default().admits("service"));
    }
}
