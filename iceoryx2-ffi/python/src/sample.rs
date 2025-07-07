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

use crate::{
    header_publish_subscribe::HeaderPublishSubscribe, parc::Parc, type_storage::TypeStorage,
};

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
pub struct Sample {
    pub(crate) value: Parc<SampleType>,
    pub payload_type_details: TypeStorage,
    pub user_header_type_details: TypeStorage,
}

#[pymethods]
impl Sample {
    #[getter]
    pub fn __payload_type_details(&self) -> Option<Py<PyAny>> {
        self.payload_type_details.clone().value
    }

    #[getter]
    pub fn __user_header_type_details(&self) -> Option<Py<PyAny>> {
        self.user_header_type_details.clone().value
    }

    #[getter]
    pub fn __slice_len(&self) -> usize {
        match &*self.value.lock() {
            SampleType::Ipc(Some(v)) => v.payload().len(),
            SampleType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "Sample::header()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns the `HeaderPublishSubscribe` of the `Sample`.
    pub fn header(&self) -> HeaderPublishSubscribe {
        match &*self.value.lock() {
            SampleType::Ipc(Some(v)) => HeaderPublishSubscribe(*v.header()),
            SampleType::Local(Some(v)) => HeaderPublishSubscribe(*v.header()),
            _ => fatal_panic!(from "Sample::header()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the user header.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            SampleType::Ipc(Some(v)) => (v.user_header() as *const CustomHeaderMarker) as usize,
            SampleType::Local(Some(v)) => (v.user_header() as *const CustomHeaderMarker) as usize,
            _ => fatal_panic!(from "Sample::user_header_ptr()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the payload.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            SampleType::Ipc(Some(v)) => (v.payload().as_ptr()) as usize,
            SampleType::Local(Some(v)) => (v.payload().as_ptr()) as usize,
            _ => fatal_panic!(from "Sample::payload_ptr()",
                "Accessing a released sample."),
        }
    }

    /// Releases the `Sample`.
    ///
    /// After this call the `Sample` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            SampleType::Ipc(ref mut v) => {
                v.take();
            }
            SampleType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
