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

#[pyclass(eq, eq_int, skip_from_py_object)]
#[derive(PartialEq, Clone, Debug)]
/// Defines the strategy a sender shall pursue when the buffer of the receiver is full
/// and the service does not overflow.
pub enum BackpressureStrategy {
    /// Retries until the receiver has consumed some
    /// data from the full buffer and there is space again
    RetryUntilDelivered,
    /// Do not deliver the data to receiver with a full buffer
    DiscardData,
}

#[pymethods]
impl BackpressureStrategy {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<iceoryx2::prelude::BackpressureStrategy> for BackpressureStrategy {
    fn from(value: iceoryx2::prelude::BackpressureStrategy) -> Self {
        match value {
            iceoryx2::prelude::BackpressureStrategy::RetryUntilDelivered => {
                BackpressureStrategy::RetryUntilDelivered
            }
            iceoryx2::prelude::BackpressureStrategy::DiscardData => {
                BackpressureStrategy::DiscardData
            }
        }
    }
}

impl From<BackpressureStrategy> for iceoryx2::prelude::BackpressureStrategy {
    fn from(value: BackpressureStrategy) -> Self {
        match value {
            BackpressureStrategy::RetryUntilDelivered => {
                iceoryx2::prelude::BackpressureStrategy::RetryUntilDelivered
            }
            BackpressureStrategy::DiscardData => {
                iceoryx2::prelude::BackpressureStrategy::DiscardData
            }
        }
    }
}
