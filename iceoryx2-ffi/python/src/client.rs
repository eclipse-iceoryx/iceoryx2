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

use crate::unique_client_id::UniqueClientId;

type IpcClient = Arc<
    iceoryx2::port::client::Client<
        crate::IpcService,
        [CustomPayloadMarker],
        CustomHeaderMarker,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
>;
type LocalClient = Arc<
    iceoryx2::port::client::Client<
        crate::LocalService,
        [CustomPayloadMarker],
        CustomHeaderMarker,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
>;

pub(crate) enum ClientType {
    Ipc(IpcClient),
    Local(LocalClient),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Client(pub(crate) ClientType);

#[pymethods]
impl Client {
    #[getter]
    /// Returns the `UniqueClientId` of the `Client`
    pub fn id(&self) -> UniqueClientId {
        match &self.0 {
            ClientType::Ipc(v) => UniqueClientId(v.id()),
            ClientType::Local(v) => UniqueClientId(v.id()),
        }
    }
}
