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

#[pyclass(str = "{0:?}")]
/// Returned by `Node.cleanup_dead_nodes()`. Contains the cleanup report of the call
/// and contains the number of dead nodes that were successfully cleaned up and how many
/// could not be cleaned up.
/// This does not have to be an error, for instance when the current process does not
/// have the permission to access the corresponding resources.
pub struct CleanupState(pub(crate) iceoryx2::node::CleanupState);

#[pymethods]
impl CleanupState {
    #[getter]
    /// The number of successful dead node cleanups
    pub fn cleanups(&self) -> usize {
        self.0.cleanups
    }

    #[getter]
    /// The number of failed dead node cleanups
    pub fn failed_cleanups(&self) -> usize {
        self.0.failed_cleanups
    }
}
