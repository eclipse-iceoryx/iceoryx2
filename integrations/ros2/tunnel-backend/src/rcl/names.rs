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

//! Validated ROS 2 names. Each type checks the relevant ROS 2 naming rules on
//! construction and holds the result as a [`CString`], so an instance is a
//! proof that the contained string is a legal name of that kind, ready to hand
//! to rcl across the FFI boundary.

use std::ffi::{CStr, CString};

use iceoryx2_log::fail;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NameError {
    /// The name is empty where a name is required.
    Empty,
    /// A token is empty or contains characters outside `[A-Za-z0-9_]`, or
    /// starts with a digit.
    InvalidToken,
    /// A namespace does not start with `/`.
    NoLeadingSlash,
}

impl core::fmt::Display for NameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NameError::{self:?}")
    }
}

impl core::error::Error for NameError {}

/// A single ROS 2 name token: non-empty, only `[A-Za-z0-9_]`, not starting with
/// a digit. This is common for all ROS 2 names.
fn is_valid_token(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

/// Whether every `/`-separated segment of `path` is a valid token. `path` must
/// not carry a leading `/`; an empty segment (from `//` or a trailing `/`)
/// fails the token check.
fn all_segments_valid(path: &str) -> bool {
    path.split('/').all(is_valid_token)
}

/// Converts an already-validated name into a [`CString`].
fn into_cstring(name: &str) -> CString {
    CString::new(name).expect("a validated ROS 2 name contains no interior nul byte")
}

/// A ROS 2 node name: a single name token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeName(CString);

impl NodeName {
    /// Creates a new ROS 2 node name from the given string.
    ///
    /// A valid node name must:
    /// - Be non-empty
    /// - Start with a letter or underscore
    /// - Contain only ASCII alphanumeric characters or underscores
    pub fn new(name: &str) -> Result<Self, NameError> {
        let origin = "NodeName::new";

        if name.is_empty() {
            fail!(
                from origin,
                with NameError::Empty,
                "Failed to create node name as it is empty"
            );
        }
        if !is_valid_token(name) {
            fail!(
                from origin,
                with NameError::InvalidToken,
                "Failed to create node name from '{}' as it is not a valid token",
                name
            );
        }

        Ok(Self(into_cstring(name)))
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

/// A ROS 2 node namespace: an absolute, `/`-separated path of name tokens. The
/// empty string and `/` both denote the root namespace.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeNamespace(CString);

impl NodeNamespace {
    /// The root namespace.
    pub fn root() -> Self {
        Self(into_cstring(""))
    }

    /// Creates a new ROS 2 namespace from the given string.
    ///
    /// A valid namespace is either:
    /// - An empty string or "/" (representing the root namespace)
    /// - An absolute path starting with '/' followed by valid name tokens separated by '/'
    pub fn new(namespace: &str) -> Result<Self, NameError> {
        let origin = "Namespace::new";

        if namespace.is_empty() || namespace == "/" {
            return Ok(Self(into_cstring(namespace)));
        }
        let path = fail!(from origin,
            when namespace.strip_prefix('/').ok_or(NameError::NoLeadingSlash),
            with NameError::NoLeadingSlash,
            "Failed to create namespace from '{}' as it does not start with '/'",
            namespace);
        if !all_segments_valid(path) {
            fail!(from origin,
                with NameError::InvalidToken,
                "Failed to create namespace from '{}' as it contains an invalid token",
                namespace
            );
        }

        Ok(Self(into_cstring(namespace)))
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

/// A ROS 2 topic name: a non-empty, `/`-separated path of name tokens, either
/// absolute (leading `/`) or relative.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicName(CString);

impl TopicName {
    /// Creates a new ROS 2 topic name from the given string.
    ///
    /// A valid topic name is a non-empty, `/`-separated path of name tokens.
    /// It can be either absolute (starting with `/`) or relative.
    ///
    /// A valid topic name must:
    /// - Be non-empty
    /// - Start with a letter or underscore
    /// - Contain only ASCII alphanumeric characters or underscores
    pub fn new(topic: &str) -> Result<Self, NameError> {
        let origin = "TopicName::new";

        if topic.is_empty() {
            fail!(
                from origin,
                with NameError::Empty,
                "Failed to create topic name as it is empty"
            );
        }

        let path = topic.strip_prefix('/').unwrap_or(topic);
        if path.is_empty() || !all_segments_valid(path) {
            fail!(
                from origin,
                with NameError::InvalidToken,
                "Failed to create topic name from '{}' as it contains an invalid token",
                topic
            );
        }

        Ok(Self(into_cstring(topic)))
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_name_accepts_valid_names() {
        for name in ["iceoryx2_tunnel", "node", "_hidden", "n0de"] {
            assert!(NodeName::new(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn node_name_rejects_invalid_names() {
        assert_eq!(NodeName::new(""), Err(NameError::Empty));
        for name in ["0node", "with/slash", "with-dash", "white space", "/node"] {
            assert_eq!(NodeName::new(name), Err(NameError::InvalidToken), "{name}");
        }
    }

    #[test]
    fn namespace_accepts_valid_namespaces() {
        for namespace in ["", "/", "/foo", "/foo/bar", "/_hidden/n0de"] {
            assert!(NodeNamespace::new(namespace).is_ok(), "{namespace}");
        }
    }

    #[test]
    fn namespace_rejects_invalid_namespaces() {
        assert_eq!(NodeNamespace::new("foo"), Err(NameError::NoLeadingSlash));
        for namespace in ["/foo/", "/foo//bar", "/0foo", "/foo/-bar"] {
            assert_eq!(
                NodeNamespace::new(namespace),
                Err(NameError::InvalidToken),
                "{namespace}"
            );
        }
    }

    #[test]
    fn topic_accepts_valid_topics() {
        for topic in ["/chatter", "/Camera/FrontRight", "chatter", "ns/topic"] {
            assert!(TopicName::new(topic).is_ok(), "{topic}");
        }
    }

    #[test]
    fn topic_rejects_invalid_topics() {
        assert_eq!(TopicName::new(""), Err(NameError::Empty));
        for topic in ["/", "/chatter/", "/0chatter", "//chatter", "with-dash"] {
            assert_eq!(
                TopicName::new(topic),
                Err(NameError::InvalidToken),
                "{topic}"
            );
        }
    }
}
