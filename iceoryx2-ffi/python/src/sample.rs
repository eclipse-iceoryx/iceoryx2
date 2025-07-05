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
use iceoryx2_bb_log::fatal_panic;
use pyo3::prelude::*;

use crate::{header_publish_subscribe::HeaderPublishSubscribe, parc::Parc};

pub(crate) enum SampleType {
    Ipc(
        Option<
            iceoryx2::sample::Sample<crate::IpcService, [CustomPayloadMarker], CustomHeaderMarker>,
        >,
    ),
    Local(
        Option<
            iceoryx2::sample::Sample<
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
/// It stores the payload and is acquired by the `Subscriber` whenever
/// it receives new data from a `Publisher` via `Subscriber::receive()`.
pub struct Sample(pub(crate) Parc<SampleType>);

#[pymethods]
impl Sample {
    #[getter]
    /// Returns the `HeaderPublishSubscribe` of the `Sample`.
    pub fn header(&self) -> HeaderPublishSubscribe {
        match &*self.0.lock() {
            SampleType::Ipc(Some(v)) => HeaderPublishSubscribe(*v.header()),
            SampleType::Local(Some(v)) => HeaderPublishSubscribe(*v.header()),
            _ => fatal_panic!(from "SampleMutUninit::header()",
                "Access of a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the user header.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.0.lock() {
            SampleType::Ipc(Some(v)) => (v.user_header() as *const CustomHeaderMarker) as usize,
            SampleType::Local(Some(v)) => (v.user_header() as *const CustomHeaderMarker) as usize,
            _ => fatal_panic!(from "SampleMutUninit::user_header_ptr()",
                "Access of a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the payload.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.0.lock() {
            SampleType::Ipc(Some(v)) => (v.payload().as_ptr()) as usize,
            SampleType::Local(Some(v)) => (v.payload().as_ptr()) as usize,
            _ => fatal_panic!(from "SampleMutUninit::user_header_ptr()",
                "Access of a released sample."),
        }
    }

    /// Releases the `SampleMutUninit`.
    ///
    /// After this call the `SampleMutUninit` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.0.lock() {
            SampleType::Ipc(ref mut v) => {
                v.take();
            }
            SampleType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
