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

use crate::{attribute::Attribute, attribute_key::AttributeKey, attribute_value::AttributeValue};

#[pyclass(str = "{0:?}", eq)]
#[derive(PartialEq)]
/// Represents a single service attribute (key-value) pair that can be defined when the service
/// is being created.
pub struct AttributeSet(pub(crate) iceoryx2::service::attribute::AttributeSet);

#[pymethods]
impl AttributeSet {
    #[getter]
    /// Returns the number of `Attribute`s stored inside the `AttributeSet`.
    pub fn number_of_attributes(&self) -> usize {
        self.0.number_of_attributes()
    }

    #[staticmethod]
    /// Returns the maximum number of `Attribute`s the `AttributeSet` can hold.
    pub fn capacity() -> usize {
        iceoryx2::service::attribute::AttributeSet::capacity()
    }

    /// Returns all `AttributeValue` that belong to a specified `AttributeKey`.
    pub fn key_values(&self, key: &AttributeKey) -> Vec<AttributeValue> {
        let mut ret_val = vec![];
        self.0.iter_key_values(&key.0, |v| {
            ret_val.push(AttributeValue(v.clone()));
            iceoryx2::prelude::CallbackProgression::Continue
        });

        ret_val
    }

    #[getter]
    /// Returns all `Attribute`s stored in the `AttributeSet`
    pub fn values(&self) -> Vec<Attribute> {
        let mut ret_val = vec![];
        for value in &*self.0 {
            ret_val.push(Attribute(value.clone()))
        }

        ret_val
    }
}
