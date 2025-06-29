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

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone, Debug)]
/// Defines the strategy a sender shall pursue when the buffer of the receiver is full
/// and the service does not overflow.
pub enum UnableToDeliverStrategy {
    /// Blocks until the receiver has consumed the
    /// data from the buffer and there is space again
    Block,
    /// Do not deliver the data.
    DiscardSample,
}

#[pymethods]
impl UnableToDeliverStrategy {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<iceoryx2::prelude::UnableToDeliverStrategy> for UnableToDeliverStrategy {
    fn from(value: iceoryx2::prelude::UnableToDeliverStrategy) -> Self {
        match value {
            iceoryx2::prelude::UnableToDeliverStrategy::Block => UnableToDeliverStrategy::Block,
            iceoryx2::prelude::UnableToDeliverStrategy::DiscardSample => {
                UnableToDeliverStrategy::DiscardSample
            }
        }
    }
}

impl From<UnableToDeliverStrategy> for iceoryx2::prelude::UnableToDeliverStrategy {
    fn from(value: UnableToDeliverStrategy) -> Self {
        match value {
            UnableToDeliverStrategy::Block => iceoryx2::prelude::UnableToDeliverStrategy::Block,
            UnableToDeliverStrategy::DiscardSample => {
                iceoryx2::prelude::UnableToDeliverStrategy::DiscardSample
            }
        }
    }
}
