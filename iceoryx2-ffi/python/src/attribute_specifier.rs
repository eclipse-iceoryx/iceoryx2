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

#[pyclass(str = "{0:?}")]
/// Represents a single service attribute (key-value) pair that can be defined when the service
/// is being created.
pub struct AttributeSpecifier(pub(crate) iceoryx2::service::attribute::AttributeSpecifier);

impl Default for AttributeSpecifier {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl AttributeSpecifier {
    #[staticmethod]
    /// Creates a new empty set of `Attribute`s
    pub fn new() -> Self {
        Self(iceoryx2::service::attribute::AttributeSpecifier::new())
    }

    /// Defines a value for a specific key. A key is allowed to have multiple values.
    pub fn define(&self, key: &AttributeKey, value: &AttributeValue) -> Self {
        let this = self.0.clone();
        let this = this.define(&key.0, &value.0);
        AttributeSpecifier(this)
    }

    #[getter]
    /// Returns the underlying `AttributeSet`
    pub fn attributes(&self) -> AttributeSet {
        AttributeSet(self.0.attributes().clone())
    }
}
