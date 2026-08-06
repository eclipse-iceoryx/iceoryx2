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

//! The set of ROS 2 topics a mapping is scoped to.

use crate::config::TopicName;

/// Trailing marker of a pattern admitting every topic under a prefix.
const WILDCARD: &str = "**";

/// One entry of an [`AllowList`].
#[derive(Debug, Clone, Eq, PartialEq)]
enum Entry {
    /// Admits one topic.
    Exact(String),
    /// Admits every topic carrying the given prefix.
    Prefixed(String),
}

impl Entry {
    /// Reads an entry from a topic name, or from a `<prefix>**` pattern.
    fn parse(pattern: &str) -> Self {
        match pattern.strip_suffix(WILDCARD) {
            Some(prefix) => Entry::Prefixed(prefix.to_string()),
            None => Entry::Exact(pattern.to_string()),
        }
    }

    /// Whether this entry admits `topic`.
    fn admits(&self, topic: &str) -> bool {
        match self {
            Entry::Exact(name) => name == topic,
            Entry::Prefixed(prefix) => topic.starts_with(prefix.as_str()),
        }
    }
}

/// The topics a mapping bridges, written as exact names or as `<prefix>/**`
/// patterns.
///
/// ```text
/// /cmd_vel        admits only /cmd_vel
/// /camera/**      admits /camera/front, /camera/rear/depth, ...
/// /**             admits every topic
/// ```
#[derive(Debug, Default, Clone)]
pub struct AllowList {
    entries: Vec<Entry>,
}

impl AllowList {
    /// Builds an allow list from topic names and patterns.
    pub fn new<S: AsRef<str>>(patterns: &[S]) -> Self {
        Self {
            entries: patterns
                .iter()
                .map(|pattern| Entry::parse(pattern.as_ref()))
                .collect(),
        }
    }

    /// Whether any entry admits `topic`.
    pub fn admits(&self, topic: &TopicName) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.admits(topic.as_str()))
    }

    /// Whether no topic was named at all.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topic(name: &str) -> TopicName {
        TopicName::new(name).expect("valid topic name")
    }

    #[test]
    fn exact_entry_admits_only_its_topic() {
        let sut = AllowList::new(&["/cmd_vel"]);

        assert!(sut.admits(&topic("/cmd_vel")));
        assert!(!sut.admits(&topic("/cmd_vel_stamped")));
        assert!(!sut.admits(&topic("/other")));
    }

    #[test]
    fn prefixed_entry_admits_every_topic_below_it() {
        let sut = AllowList::new(&["/camera/**"]);

        assert!(sut.admits(&topic("/camera/front")));
        assert!(sut.admits(&topic("/camera/rear/depth")));
        assert!(!sut.admits(&topic("/lidar/front")));
    }

    #[test]
    fn root_pattern_admits_every_topic() {
        let sut = AllowList::new(&["/**"]);

        assert!(sut.admits(&topic("/chatter")));
        assert!(sut.admits(&topic("/camera/front")));
    }

    #[test]
    fn entries_accumulate() {
        let sut = AllowList::new(&["/cmd_vel", "/camera/**"]);

        assert!(sut.admits(&topic("/cmd_vel")));
        assert!(sut.admits(&topic("/camera/front")));
        assert!(!sut.admits(&topic("/lidar")));
    }

    #[test]
    fn empty_list_admits_nothing() {
        let sut = AllowList::default();

        assert!(sut.is_empty());
        assert!(!sut.admits(&topic("/chatter")));
    }
}
