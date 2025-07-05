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

use crate::parc::Parc;

pub(crate) enum SampleMutUninitType {
    Ipc(
        iceoryx2::sample_mut_uninit::SampleMutUninit<
            crate::IpcService,
            [MaybeUninit<CustomPayloadMarker>],
            CustomHeaderMarker,
        >,
    ),
    Local(
        iceoryx2::sample_mut_uninit::SampleMutUninit<
            crate::LocalService,
            [MaybeUninit<CustomPayloadMarker>],
            CustomHeaderMarker,
        >,
    ),
}

#[pyclass]
pub struct SampleMutUninit(pub(crate) Parc<SampleMutUninitType>);
