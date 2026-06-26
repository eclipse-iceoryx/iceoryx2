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
//! construction and holds the result as a C string, so an instance is a proof
//! that the contained string is a legal name of that kind, ready to hand to rcl
//! across the FFI boundary.

use std::borrow::Cow;
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

/// Stores an already-validated name as an owned C string.
fn owned(name: &str) -> Cow<'static, CStr> {
    Cow::Owned(CString::new(name).expect("a validated ROS 2 name contains no interior nul byte"))
}

/// A ROS 2 node name: a single name token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeName(Cow<'static, CStr>);

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

        Ok(Self(owned(name)))
    }

    /// Wraps an already-valid node name without re-checking it.
    ///
    /// The caller must guarantee `name` is a well-formed ROS 2 node name.
    pub fn new_unchecked(name: &str) -> Self {
        Self(owned(name))
    }

    /// Wraps a static, already-valid node name at compile time.
    pub const fn new_static_unchecked(name: &'static CStr) -> Self {
        Self(Cow::Borrowed(name))
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

/// A ROS 2 node namespace: an absolute, `/`-separated path of name tokens. The
/// empty string and `/` both denote the root namespace.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeNamespace(Cow<'static, CStr>);

impl NodeNamespace {
    /// The root namespace.
    pub const fn root() -> Self {
        Self(Cow::Borrowed(c""))
    }

    /// Creates a new ROS 2 namespace from the given string.
    ///
    /// A valid namespace is either:
    /// - An empty string or "/" (representing the root namespace)
    /// - An absolute path starting with '/' followed by valid name tokens separated by '/'
    pub fn new(namespace: &str) -> Result<Self, NameError> {
        let origin = "Namespace::new";

        if namespace.is_empty() || namespace == "/" {
            return Ok(Self(owned(namespace)));
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

        Ok(Self(owned(namespace)))
    }

    /// Wraps an already-valid namespace without re-checking it.
    ///
    /// The caller must guarantee `namespace` is a well-formed ROS 2 node namespace.
    pub fn new_unchecked(namespace: &str) -> Self {
        Self(owned(namespace))
    }

    /// Wraps a static, already-valid namespace at compile time.
    pub const fn new_static_unchecked(namespace: &'static CStr) -> Self {
        Self(Cow::Borrowed(namespace))
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

/// A ROS 2 topic name: a non-empty, `/`-separated path of name tokens, either
/// absolute (leading `/`) or relative.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TopicName(Cow<'static, CStr>);

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

        Ok(Self(owned(topic)))
    }

    /// Wraps an already-valid topic name without re-checking it.
    ///
    /// The caller must guarantee `topic` is a well-formed ROS 2 topic name,
    /// e.g. it came straight from an rcl graph query.
    pub fn new_unchecked(topic: &str) -> Self {
        Self(owned(topic))
    }

    /// Wraps a static, already-valid topic name at compile time.
    pub const fn new_static_unchecked(topic: &'static CStr) -> Self {
        Self(Cow::Borrowed(topic))
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }

    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0
            .to_str()
            .expect("a validated ROS 2 name is valid UTF-8")
    }
}

/// A ROS 2 message type name of the form `package/msg/Message`, e.g.
/// `std_msgs/msg/String`. The `package` and `Message` parts are valid name
/// tokens.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypeName(Cow<'static, CStr>);

impl TypeName {
    /// Creates a new ROS 2 message type name from the given string.
    ///
    /// A valid type name has the form `package/msg/Message`, where `package`
    /// and `Message` are each a valid name token.
    pub fn new(type_name: &str) -> Result<Self, NameError> {
        let origin = "TypeName::new";

        if type_name.is_empty() {
            fail!(
                from origin,
                with NameError::Empty,
                "Failed to create type name as it is empty"
            );
        }

        let mut segments = type_name.split('/');
        let well_formed = matches!(
            (segments.next(), segments.next(), segments.next(), segments.next()),
            (Some(package), Some("msg"), Some(message), None)
                if is_valid_token(package) && is_valid_token(message)
        );
        if !well_formed {
            fail!(
                from origin,
                with NameError::InvalidToken,
                "Failed to create type name from '{}' as it is not of the form 'package/msg/Message'",
                type_name
            );
        }

        Ok(Self(owned(type_name)))
    }

    /// Wraps an already-valid type name without re-checking it.
    ///
    /// The caller must guarantee `type_name` is a well-formed ROS 2 type name,
    /// e.g. it came straight from an rcl graph query.
    pub fn new_unchecked(type_name: &str) -> Self {
        Self(owned(type_name))
    }

    /// Wraps a static, already-valid type name at compile time.
    pub const fn new_static_unchecked(type_name: &'static CStr) -> Self {
        Self(Cow::Borrowed(type_name))
    }

    /// The type name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0
            .to_str()
            .expect("a validated ROS 2 name is valid UTF-8")
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

    #[test]
    fn type_name_accepts_valid_types() {
        for type_name in [
            "std_msgs/msg/String",
            "geometry_msgs/msg/Twist",
            "my_pkg/msg/Vector3",
        ] {
            assert!(TypeName::new(type_name).is_ok(), "{type_name}");
        }
    }

    #[test]
    fn type_name_rejects_invalid_types() {
        assert_eq!(TypeName::new(""), Err(NameError::Empty));
        for type_name in [
            "std_msgs/String",
            "std_msgs/msg/String/extra",
            "std_msgs/srv/Thing",
            "/msg/String",
            "std_msgs/msg/",
            "std_msgs/msg/0String",
        ] {
            assert_eq!(
                TypeName::new(type_name),
                Err(NameError::InvalidToken),
                "{type_name}"
            );
        }
    }

    #[test]
    fn constructors_round_trip_and_agree() {
        const STATIC: TopicName = TopicName::new_static_unchecked(c"/chatter");
        let checked = TopicName::new("/chatter").unwrap();
        let unchecked = TopicName::new_unchecked("/chatter");

        assert_eq!(checked.as_str(), "/chatter");
        assert_eq!(checked.as_c_str(), c"/chatter");
        assert_eq!(checked, unchecked);
        assert_eq!(checked, STATIC);
    }

    #[test]
    fn root_namespace_is_empty() {
        assert_eq!(NodeNamespace::root().as_c_str(), c"");
    }
}
