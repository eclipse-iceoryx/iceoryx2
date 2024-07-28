// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

//! Attributes can be defined for a [`crate::service::Service`]. They define features that do not
//! change during the lifetime of the [`crate::service::Service`] and are accessible by anyone that
//! is allowed to open the [`crate::service::Service`].
//!
//! ## Create Service With Attributes
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service_creator = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .create_with_attributes(
//!         // all attributes that are defined when creating a new service are stored in the
//!         // static config of the service
//!         &AttributeSpecifier::new()
//!             .define("some attribute key", "some attribute value")
//!             .define("some attribute key", "another attribute value for the same key")
//!             .define("another key", "another value")
//!     )?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Open Service With Attribute Requirements
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! use iceoryx2::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! let service_open = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_with_attributes(
//!         // All attributes that are defined when opening a new service interpreted as
//!         // requirements.
//!         // If a attribute key as either a different value or is not set at all, the service
//!         // cannot be opened. If not specific attributes are required one can skip them completely.
//!         &AttributeVerifier::new()
//!             .require("another key", "another value")
//!             .require_key("some attribute key")
//!     )?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## List Attributes Of A Service
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! use iceoryx2::config::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open()?;
//!
//! for attribute in service.attributes().iter() {
//!     println!("key {}, value {}", attribute.key(), attribute.value());
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## List Attributes Of All Services In Discovery
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let services = ipc::Service::list(Config::global_config(), |service| {
//!     println!("\n{:#?}", &service.static_details.attributes());
//!     CallbackProgression::Continue
//! })?;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Represents a single service attribute (key-value) pair that can be defined when the service
/// is being created.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, PartialOrd, Ord)]
pub struct Attribute {
    key: String,
    value: String,
}

impl Attribute {
    /// Acquires the service attribute key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Acquires the service attribute value
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Represents the set of [`Attribute`]s that are defined when the [`crate::service::Service`]
/// is created.
pub struct AttributeSpecifier(pub(crate) AttributeSet);

impl Default for AttributeSpecifier {
    fn default() -> Self {
        Self(AttributeSet::new())
    }
}

impl AttributeSpecifier {
    /// Creates a new empty set of [`Attribute`]s
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines a value for a specific key. A key is allowed to have multiple values.
    pub fn define(mut self, key: &str, value: &str) -> Self {
        self.0.add(key, value);
        self
    }

    /// Returns the underlying [`AttributeSet`]
    pub fn attributes(&self) -> &AttributeSet {
        &self.0
    }
}

/// Represents the set of [`Attribute`]s that are required when the [`crate::service::Service`]
/// is opened.
#[derive(Debug)]
pub struct AttributeVerifier {
    attribute_set: AttributeSet,
    required_keys: Vec<String>,
}

impl Default for AttributeVerifier {
    fn default() -> Self {
        Self {
            attribute_set: AttributeSet::new(),
            required_keys: Vec::new(),
        }
    }
}

impl AttributeVerifier {
    /// Creates a new empty set of [`Attribute`]s
    pub fn new() -> Self {
        Self::default()
    }

    /// Requires a value for a specific key. A key is allowed to have multiple values.
    pub fn require(mut self, key: &str, value: &str) -> Self {
        self.attribute_set.add(key, value);
        self
    }

    /// Requires that a specific key is defined.
    pub fn require_key(mut self, key: &str) -> Self {
        self.required_keys.push(key.into());
        self
    }

    /// Returns the underlying required [`AttributeSet`]
    pub fn attributes(&self) -> &AttributeSet {
        &self.attribute_set
    }

    /// Returns the underlying required keys
    pub fn keys(&self) -> &Vec<String> {
        &self.required_keys
    }

    /// Verifies if the [`AttributeSet`] contains all required keys and key-value pairs.
    pub fn verify_requirements(&self, rhs: &AttributeSet) -> Result<(), &str> {
        let is_subset = |lhs: Vec<&str>, rhs: Vec<&str>| lhs.iter().all(|v| rhs.contains(v));

        for attribute in self.attributes().iter() {
            let lhs_values = self.attribute_set.get(&attribute.key);
            let rhs_values = rhs.get(&attribute.key);

            if !is_subset(lhs_values, rhs_values) {
                return Err(&attribute.key);
            }
        }

        for key in self.keys() {
            if rhs.get(key).is_empty() {
                return Err(key);
            }
        }

        Ok(())
    }
}

/// Represents all service attributes. They can be set when the service is created.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct AttributeSet(Vec<Attribute>);

impl Deref for AttributeSet {
    type Target = [Attribute];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl AttributeSet {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn add(&mut self, key: &str, value: &str) {
        self.0.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
        self.0.sort();
    }

    /// Returns all values to a specific key
    pub fn get(&self, key: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|p| p.key == key)
            .map(|p| p.value.as_str())
            .collect()
    }
}
