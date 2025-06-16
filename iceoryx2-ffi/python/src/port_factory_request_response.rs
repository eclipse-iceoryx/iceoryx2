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

use iceoryx2::prelude::{ipc, local};
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

pub(crate) enum PortFactoryRequestResponseType {
    Ipc(
        iceoryx2::service::port_factory::request_response::PortFactory<
            ipc::Service,
            CustomPayloadMarker,
            CustomHeaderMarker,
            CustomPayloadMarker,
            CustomHeaderMarker,
        >,
    ),
    Local(
        iceoryx2::service::port_factory::request_response::PortFactory<
            local::Service,
            CustomPayloadMarker,
            CustomHeaderMarker,
            CustomPayloadMarker,
            CustomHeaderMarker,
        >,
    ),
}

#[pyclass]
pub struct PortFactoryRequestResponse(pub(crate) PortFactoryRequestResponseType);
