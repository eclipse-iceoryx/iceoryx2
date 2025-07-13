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

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::{parc::Parc, type_storage::TypeStorage};

type IpcResponse =
    iceoryx2::response::Response<crate::IpcService, [CustomPayloadMarker], CustomHeaderMarker>;
type LocalResponse =
    iceoryx2::response::Response<crate::LocalService, [CustomPayloadMarker], CustomHeaderMarker>;

pub(crate) enum ResponseType {
    Ipc(Option<IpcResponse>),
    Local(Option<LocalResponse>),
}

#[pyclass]
pub struct Response {
    pub(crate) value: Parc<ResponseType>,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}
