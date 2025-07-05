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

use std::sync::Arc;

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::unique_publisher_id::UniquePublisherId;

pub(crate) enum PublisherType {
    Ipc(
        Arc<
            iceoryx2::port::publisher::Publisher<
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Arc<
            iceoryx2::port::publisher::Publisher<
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Publisher(pub(crate) PublisherType);

#[pymethods]
impl Publisher {
    #[getter]
    /// Returns the `UniquePublisherId` of the `Publisher`
    pub fn id(&self) -> UniquePublisherId {
        match &self.0 {
            PublisherType::Ipc(v) => UniquePublisherId(v.id()),
            PublisherType::Local(v) => UniquePublisherId(v.id()),
        }
    }
}
