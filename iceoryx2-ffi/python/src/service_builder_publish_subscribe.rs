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

pub(crate) enum ServiceBuilderPublishSubscribeType {
    Ipc(
        iceoryx2::service::builder::publish_subscribe::Builder<
            CustomPayloadMarker,
            CustomHeaderMarker,
            ipc::Service,
        >,
    ),
    Local(
        iceoryx2::service::builder::publish_subscribe::Builder<
            CustomPayloadMarker,
            CustomHeaderMarker,
            local::Service,
        >,
    ),
}

#[pyclass]
pub struct ServiceBuilderPublishSubscribe(pub(crate) ServiceBuilderPublishSubscribeType);
