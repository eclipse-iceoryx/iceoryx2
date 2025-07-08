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

use crate::{node_id::NodeId, unique_publisher_id::UniquePublisherId};

#[pyclass(eq)]
#[derive(PartialEq, Eq)]
/// Sample header used by `MessagingPattern::PublishSubscribe`
pub struct HeaderPublishSubscribe(pub(crate) iceoryx2::service::header::publish_subscribe::Header);

#[pymethods]
impl HeaderPublishSubscribe {
    #[getter]
    /// Returns the `NodeId` of the source node that published the `Sample`.
    pub fn node_id(&self) -> NodeId {
        NodeId(self.0.node_id())
    }

    #[getter]
    /// Returns the `UniquePublisherId` of the source `Publisher`.
    pub fn publisher_id(&self) -> UniquePublisherId {
        UniquePublisherId(self.0.publisher_id())
    }

    #[getter]
    /// Returns how many elements are stored inside the `Sample`'s payload.
    pub fn number_of_elements(&self) -> u64 {
        self.0.number_of_elements()
    }
}
