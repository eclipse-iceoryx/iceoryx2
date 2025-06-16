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

use crate::{attribute_key::AttributeKey, attribute_value::AttributeValue};
use pyo3::prelude::*;

#[pyclass(str = "{0:?}", eq)]
#[derive(PartialEq)]
/// Represents a single service attribute (key-value) pair that can be defined when the service
/// is being created.
pub struct Attribute(pub(crate) iceoryx2::service::attribute::Attribute);

#[pymethods]
impl Attribute {
    #[staticmethod]
    /// Creates an attribute instance
    pub fn new(key: &AttributeKey, value: &AttributeValue) -> Self {
        Self(iceoryx2::service::attribute::Attribute::new(
            &key.0, &value.0,
        ))
    }

    #[getter]
    /// Acquires the service attribute key
    pub fn key(&self) -> AttributeKey {
        AttributeKey(self.0.key().clone())
    }

    #[getter]
    /// Acquires the service attribute value
    pub fn value(&self) -> AttributeValue {
        AttributeValue(self.0.value().clone())
    }
}
