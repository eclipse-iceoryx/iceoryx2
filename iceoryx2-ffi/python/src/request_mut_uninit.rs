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

use core::mem::MaybeUninit;

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::{parc::Parc, type_storage::TypeStorage};

type IpcRequestMutUninit = iceoryx2::request_mut_uninit::RequestMutUninit<
    crate::IpcService,
    [MaybeUninit<CustomPayloadMarker>],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalRequestMutUninit = iceoryx2::request_mut_uninit::RequestMutUninit<
    crate::LocalService,
    [MaybeUninit<CustomPayloadMarker>],
    CustomHeaderMarker,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum RequestMutUninitType {
    Ipc(Option<IpcRequestMutUninit>),
    Local(Option<LocalRequestMutUninit>),
}

#[pyclass]
/// A version of the `RequestMut` where the payload is not initialized which allows
/// true zero copy usage. To send a `RequestMutUninit` it must be first initialized
/// and converted into `RequestMut` with `RequestMutUninit::assume_init()`.
pub struct RequestMutUninit {
    pub(crate) value: Parc<RequestMutUninitType>,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}
