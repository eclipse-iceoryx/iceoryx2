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

use crate::node_name::NodeName;

#[pyclass(str = "{0:?}")]
/// Represent the name for a `Node`.
pub struct NodeBuilder(iceoryx2::prelude::NodeBuilder);

#[pymethods]
impl NodeBuilder {
    #[staticmethod]
    pub fn new() -> Self {
        Self {
            0: iceoryx2::prelude::NodeBuilder::new(),
        }
    }

    pub fn name(&mut self, value: &NodeName) -> Self {
        let this = self.0.clone();
        let this = this.name(&value.0);
        Self(this)
    }
}
