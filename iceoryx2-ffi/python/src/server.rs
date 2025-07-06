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

use crate::unique_server_id::UniqueServerId;

type IpcServer = Arc<
    iceoryx2::port::server::Server<
        crate::IpcService,
        [CustomPayloadMarker],
        CustomHeaderMarker,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
>;
type LocalServer = Arc<
    iceoryx2::port::server::Server<
        crate::LocalService,
        [CustomPayloadMarker],
        CustomHeaderMarker,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
>;

pub(crate) enum ServerType {
    Ipc(IpcServer),
    Local(LocalServer),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Server(pub(crate) ServerType);

#[pymethods]
impl Server {
    #[getter]
    /// Returns the `UniqueServerId` of the `Server`
    pub fn id(&self) -> UniqueServerId {
        match &self.0 {
            ServerType::Ipc(v) => UniqueServerId(v.id()),
            ServerType::Local(v) => UniqueServerId(v.id()),
        }
    }
}
