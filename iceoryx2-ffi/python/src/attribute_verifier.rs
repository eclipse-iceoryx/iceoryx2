// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use pyo3::prelude::*;

use crate::{
    attribute_key::AttributeKey, attribute_set::AttributeSet, attribute_value::AttributeValue,
};
use iceoryx2::prelude::SemanticString;

#[pyclass(str = "{0:?}")]
/// Represents a single service attribute (key-value) pair that can be defined when the service
/// is being created.
pub struct AttributeVerifier(pub(crate) iceoryx2::service::attribute::AttributeVerifier);

impl Default for AttributeVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl AttributeVerifier {
    #[staticmethod]
    /// Creates a new empty set of `Attribute`s
    pub fn new() -> Self {
        Self(iceoryx2::service::attribute::AttributeVerifier::new())
    }

    /// Requires a value for a specific key. A key is allowed to have multiple values.
    pub fn require(&self, key: &AttributeKey, value: &AttributeValue) -> Self {
        let this = self.0.clone();
        let this = this.require(&key.0, &value.0);
        AttributeVerifier(this)
    }

    /// Requires that a specific key is defined.
    pub fn require_key(&self, key: &AttributeKey) -> Self {
        let this = self.0.clone();
        let this = this.require_key(&key.0);
        AttributeVerifier(this)
    }

    #[getter]
    /// Returns the underlying required `AttributeSet`
    pub fn required_attributes(&self) -> AttributeSet {
        AttributeSet(self.0.required_attributes().clone())
    }

    #[getter]
    /// Returns the underlying required keys
    pub fn required_keys(&self) -> Vec<AttributeKey> {
        let mut ret_val = vec![];
        for key in self.0.required_keys() {
            ret_val.push(AttributeKey(key.clone()));
        }
        ret_val
    }

    /// Verifies if the `AttributeSet` contains all required keys and key-value pairs. If it does
    /// not satisfy the requirements it returns the first
    pub fn verify_requirements(&self, rhs: &AttributeSet) -> Option<AttributeKey> {
        match self.0.verify_requirements(&rhs.0) {
            Ok(()) => None,
            Err(e) => Some(AttributeKey(
                iceoryx2::service::attribute::AttributeKey::new(e.as_bytes()).unwrap(),
            )),
        }
    }
}
