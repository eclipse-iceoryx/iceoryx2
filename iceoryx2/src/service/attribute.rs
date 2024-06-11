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
    /// Acquires the service property key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Acquires the service property value
    pub fn value(&self) -> &str {
        &self.value
    }
}

pub struct DefinedAttributes(pub(crate) AttributeSet);

impl DefinedAttributes {
    pub fn new() -> Self {
        Self(AttributeSet::new())
    }

    pub fn define(mut self, key: &str, value: &str) -> Self {
        self.0.add(key, value);
        self
    }

    pub fn attributes(&self) -> &AttributeSet {
        &self.0
    }
}

pub struct RequiredAttributes(pub(crate) AttributeSet);

impl RequiredAttributes {
    pub fn new() -> Self {
        Self(AttributeSet::new())
    }

    pub fn require(mut self, key: &str, value: &str) -> Self {
        self.0.add(key, value);
        self
    }

    pub fn attributes(&self) -> &AttributeSet {
        &self.0
    }
}

/// Represents all service properties. They can be set when the service is created.
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

    pub(crate) fn is_compatible_to(&self, rhs: &Self) -> Result<(), &str> {
        let is_subset = |lhs: Vec<&str>, rhs: Vec<&str>| lhs.iter().all(|v| rhs.contains(v));

        for property in &self.0 {
            let lhs_values = self.get(&property.key);
            let rhs_values = rhs.get(&property.key);

            if !is_subset(lhs_values, rhs_values) {
                return Err(&property.key);
            }
        }

        Ok(())
    }
}
