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

use crate::{type_name::TypeName, type_variant::TypeVariant};
use iceoryx2::testing;

#[pyclass(str = "{0:?}", eq)]
/// Contains all type details required to connect to a `Service`
#[derive(PartialEq)]
pub struct TypeDetail(
    pub(crate) iceoryx2::service::static_config::message_type_details::TypeDetail,
);

impl Default for TypeDetail {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl TypeDetail {
    #[staticmethod]
    /// Creates a new `TypeDetail` for the unit type. Meaning size == 0, alignment == 1
    pub fn new() -> Self {
        Self(
            iceoryx2::service::static_config::message_type_details::TypeDetail::new::<()>(
                iceoryx2::service::static_config::message_type_details::TypeVariant::FixedSize,
            ),
        )
    }

    /// Defines the `TypeVariant` of the defined type. `TypeVariant::FixedSize` if the type has
    /// always the same size like an `uint64_t` or `TypeVariant::Dynamic` when it is a dynamic
    /// array or vector
    pub fn type_variant(&self, value: &TypeVariant) -> Self {
        let mut this = self.0.clone();

        testing::type_detail_set_variant(&mut this, (value.clone()).into());
        Self(this)
    }

    /// Sets the unique `TypeName` of the type
    pub fn type_name(&self, name: &TypeName) -> Self {
        let mut this = self.0.clone();
        testing::type_detail_set_name(&mut this, name.0);
        Self(this)
    }

    /// Sets the size of the type
    pub fn size(&self, size: usize) -> Self {
        let mut this = self.0.clone();
        testing::type_detail_set_size(&mut this, size);
        Self(this)
    }

    /// Sets the alignment of the type
    pub fn alignment(&self, alignment: usize) -> Self {
        let mut this = self.0.clone();
        testing::type_detail_set_alignment(&mut this, alignment);
        Self(this)
    }
}
