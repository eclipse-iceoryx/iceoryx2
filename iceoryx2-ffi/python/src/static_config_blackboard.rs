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

use crate::type_detail::TypeDetail;

#[pyclass]
/// The static configuration of an `MessagingPattern::Blackboard` based `Service`. Contains
/// all parameters that do not change during the lifetime of a `Service`.
pub struct StaticConfigBlackboard(
    pub(crate) iceoryx2::service::static_config::blackboard::StaticConfig,
);

#[pymethods]
impl StaticConfigBlackboard {
    #[getter]
    /// Returns the maximum supported amount of `Node`s that can open the `Service` in parallel.
    pub fn max_nodes(&self) -> usize {
        self.0.max_nodes()
    }

    #[getter]
    /// Returns the maximum supported amount of `Reader` ports
    pub fn max_readers(&self) -> usize {
        self.0.max_readers()
    }

    #[getter]
    /// Returns the type details of the `Service`.
    pub fn type_details(&self) -> TypeDetail {
        TypeDetail(self.0.type_details().clone())
    }
}
