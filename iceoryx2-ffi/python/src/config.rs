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

use crate::file_name::FileName;
use crate::path::Path;
use pyo3::prelude::*;

#[pyclass(name = "Node")]
pub struct Node {
    value: iceoryx2::config::Node,
}

#[pymethods]
impl Node {
    #[getter]
    pub fn cleanup_dead_nodes_on_creation(&self) -> bool {
        self.value.cleanup_dead_nodes_on_creation
    }

    #[setter]
    fn set_cleanup_dead_nodes_on_creation(&mut self, value: bool) {
        self.value.cleanup_dead_nodes_on_creation = value
    }
}
