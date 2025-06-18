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

#[pyclass(str = "{0:?}", eq)]
/// Contains all type details required to connect to a `Service`
#[derive(PartialEq)]
pub struct TypeDetail(
    pub(crate) iceoryx2::service::static_config::message_type_details::TypeDetail,
);

#[pymethods]
impl TypeDetail {
    #[staticmethod]
    pub fn new() -> Self {
        Self(
            iceoryx2::service::static_config::message_type_details::TypeDetail::__internal_new::<()>(
                iceoryx2::service::static_config::message_type_details::TypeVariant::FixedSize,
            ),
        )
    }

    pub fn type_variant(&self, value: &TypeVariant) -> Self {
        let mut this = self.0.clone();
        this.variant = (value.clone()).into();
        Self(this)
    }

    pub fn type_name(&self, name: &TypeName) -> Self {
        let mut this = self.0.clone();
        this.type_name = name.0.clone();
        Self(this)
    }

    pub fn size(&self, size: usize) -> Self {
        let mut this = self.0.clone();
        this.size = size;
        Self(this)
    }

    pub fn alignment(&self, alignment: usize) -> Self {
        let mut this = self.0.clone();
        this.alignment = alignment;
        Self(this)
    }
}
