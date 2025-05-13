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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let services = ipc::Service::list(Config::global_config(), |service| {
//!     println!("\n{:#?}", &service.static_details.attributes());
//!     CallbackProgression::Continue
//! })?;
//! # Ok(())
//! # }
//! ```

use core::ops::Deref;

use iceoryx2_bb_container::{byte_string::FixedSizeByteString, vec::FixedSizeVec};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_log::fatal_panic;
use serde::{Deserialize, Serialize};

use crate::constants::{MAX_ATTRIBUTES, MAX_ATTRIBUTE_KEY_LENGTH, MAX_ATTRIBUTE_VALUE_LENGTH};

type AttributeKeyString = FixedSizeByteString<MAX_ATTRIBUTE_KEY_LENGTH>;
type AttributeValueString = FixedSizeByteString<MAX_ATTRIBUTE_VALUE_LENGTH>;

type KeyStorage = FixedSizeVec<AttributeKeyString, MAX_ATTRIBUTES>;
type AttributeStorage = FixedSizeVec<Attribute, MAX_ATTRIBUTES>;

/// Represents a single service attribute (key-value) pair that can be defined when the service
/// is being created.
#[derive(Debug, Eq, PartialEq, Clone, PartialOrd, Ord, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct Attribute {
    key: AttributeKeyString,
    value: AttributeValueString,
}

/// Errors that can occur when creating an [`Attribute`].
pub enum AttributeError{
    /// The attribute key exceeds the maximum allowed length.
    KeyTooLong,
    /// The attribute value exceeds the maximum allowed length.
    ValueTooLong,
}

impl Attribute {
    /// Creates an attribute instance
    pub fn new(key: &str, value: &str) -> Result<Self, AttributeError> {
        Ok(Self {
            key: AttributeKeyString::try_from(key).map_err(|_| AttributeError::KeyTooLong)?,
            value: AttributeValueString::try_from(value).map_err(|_| AttributeError::ValueTooLong)?
        })
    }

    /// Acquires the service attribute key
    pub fn key(&self) -> &str {
        fatal_panic!(from self, 
             when self.key.as_str(),
             "This should never happen! The underlying attribute key does not contain a valid UTF-8 string.")
    }

    /// Acquires the service attribute value
    pub fn value(&self) -> &str {
        fatal_panic!(from self, 
             when self.value.as_str(),
             "This should never happen! The underlying attribute value does not contain a valid UTF-8 string.")
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
    required_attributes: AttributeSet,
    required_keys: KeyStorage,
}

impl Default for AttributeVerifier {
    fn default() -> Self {
        Self {
            required_attributes: AttributeSet::new(),
            required_keys: KeyStorage::new(),
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
        self.required_attributes.add(key, value);
        self
    }

    /// Requires that a specific key is defined.
    pub fn require_key(mut self, key: &str) -> Self {
        self.required_keys.push(
            fatal_panic!(
                from "AttributeVerifier::require_key", 
                when key.try_into(), 
                "Attempted to require an attribute key that does not fit into the underlying FixedSizeByteString")
        );
        self
    }

    /// Returns the underlying required [`AttributeSet`]
    pub fn required_attributes(&self) -> &AttributeSet {
        &self.required_attributes
    }

    /// Returns the underlying required keys
    pub fn required_keys(&self) -> &[AttributeKeyString] {
        self.required_keys.as_slice()
    }

    /// Verifies if the [`AttributeSet`] contains all required keys and key-value pairs.
    pub fn verify_requirements(&self, rhs: &AttributeSet) -> Result<(), &str> {
        // Implementation utilizes nested loops, however since MAX_ATTRIBUTES is small and
        // the method is not expected to be used in a hot path, performance should be fine.

        // Check if the required key-value pair exists in the target AttributeSet.
        for attribute in self.required_attributes().iter() {
            let key = &attribute.key();
            let value = &attribute.value();

            let attribute_present = rhs
                .iter()
                .any(|attr| attr.key == *key && attr.value == *value);

            if !attribute_present {
                return Err(key);
            }
        }

        // Ensure keys without values are also present in the target AttributeSet.
        for key in self.required_keys() {
            let key_exists = rhs.iter().any(|attr| attr.key == *key);

            if !key_exists {
                let key_str = fatal_panic!(from self, 
                    when key.as_str(),
                    "This should never happen! The underlying attribute key does not contain a valid UTF-8 string.");
                return Err(key_str);
            }
        }

        Ok(())
    }
}

/// Represents all service attributes. They can be set when the service is created.
#[derive(Debug, Eq, PartialEq, Clone, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct AttributeSet(AttributeStorage);

impl Deref for AttributeSet {
    type Target = [Attribute];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl AttributeSet {
    pub(crate) fn new() -> Self {
        Self(AttributeStorage::new())
    }

    pub(crate) fn add(&mut self, key: &str, value: &str) {
        self.0.push(
            fatal_panic!(
                from "AttributeSet::add(key: &str, value: &str)", 
                when Attribute::new(key, value), 
                "Attribute key or value exceeds maximum capacity")
        );
        self.0.sort();
    }

    /// Returns the number of [`Attribute`]s stored inside the [`AttributeSet`].
    pub fn number_of_attributes(&self) -> usize {
        self.iter().len()
    }

    /// Returns the number of values stored under a specific key. If the key does not exist it
    /// returns 0.
    pub fn number_of_key_values(&self, key: &str) -> usize {
        self.iter().filter(|element| element.key() == key).count()
    }

    /// Returns a value of a key at a specific index. The index enumerates the values of the key
    /// if the key has multiple values. The values are always stored at the same position during
    /// the lifetime of the service but they can change when the process is recreated by another
    /// process when the system restarts.
    /// If the key does not exist or it does not have a value at the specified index, it returns
    /// [`None`].
    pub fn key_value(&self, key: &str, idx: usize) -> Option<&str> {
        self.0
            .iter()
            .filter(|attr| attr.key == key)
            .map(|attr| attr.value())
            .nth(idx)
    }

    /// Iterates over all values of a specific key
    pub fn iter_key_values<F: FnMut(&str) -> CallbackProgression>(
        &self,
        key: &str,
        mut callback: F,
    ) {
        for element in self.iter() {
            if element.key != key {
                continue;
            }

            if callback(element.value()) == CallbackProgression::Stop {
                break;
            }
        }
    }
}
