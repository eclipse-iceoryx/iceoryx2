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
    error::SendError, header_publish_subscribe::HeaderPublishSubscribe, parc::Parc,
    type_storage::TypeStorage,
};

pub(crate) enum SampleMutType {
    Ipc(
        Option<
            iceoryx2::sample_mut::SampleMut<
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Option<
            iceoryx2::sample_mut::SampleMut<
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
/// Acquired by a `Publisher` via
///  * `Publisher::loan()`,
///  * `Publisher::loan_slice()`
///
/// It stores the payload that will be sent
/// to all connected `Subscriber`s. If the `SampleMut` is not sent
/// it will release the loaned memory when going out of scope.
pub struct SampleMut {
    pub(crate) value: Parc<SampleMutType>,
    pub(crate) payload_type_details: TypeStorage,
    pub(crate) user_header_type_details: TypeStorage,
}

#[pymethods]
impl SampleMut {
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
            SampleMutType::Ipc(Some(v)) => v.payload().len(),
            SampleMutType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "Sample::header()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns the `HeaderPublishSubscribe` of the `Sample`.
    pub fn header(&self) -> HeaderPublishSubscribe {
        match &*self.value.lock() {
            SampleMutType::Ipc(Some(v)) => HeaderPublishSubscribe(*v.header()),
            SampleMutType::Local(Some(v)) => HeaderPublishSubscribe(*v.header()),
            _ => fatal_panic!(from "SampleMut::header()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the user header.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            SampleMutType::Ipc(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            SampleMutType::Local(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            _ => fatal_panic!(from "SampleMut::user_header_ptr()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the payload.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            SampleMutType::Ipc(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            SampleMutType::Local(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            _ => fatal_panic!(from "SampleMut::user_header_ptr()",
                "Accessing a released sample."),
        }
    }

    /// Releases the `SampleMut`.
    ///
    /// After this call the `SampleMut` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            SampleMutType::Ipc(ref mut v) => {
                v.take();
            }
            SampleMutType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    /// Send a previously loaned `Publisher::loan_uninit()` `SampleMut` to all connected
    /// `Subscriber`s of the service.
    ///
    /// On success the number of `Subscriber`s that received
    /// the data is returned, otherwise a `SendError` is emitted describing the failure.
    pub fn send(&self) -> PyResult<usize> {
        match &mut *self.value.lock() {
            SampleMutType::Ipc(ref mut v) => {
                let sample = v.take().unwrap();
                Ok(sample
                    .send()
                    .map_err(|e| SendError::new_err(format!("{e:?}")))?)
            }
            SampleMutType::Local(ref mut v) => {
                let sample = v.take().unwrap();
                Ok(sample
                    .send()
                    .map_err(|e| SendError::new_err(format!("{e:?}")))?)
            }
        }
    }
}
