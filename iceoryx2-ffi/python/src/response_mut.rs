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
    error::SendError, parc::Parc, response_header::ResponseHeader, type_storage::TypeStorage,
};

type IpcResponseMut = iceoryx2::response_mut::ResponseMut<
    crate::IpcService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;
type LocalResponseMut = iceoryx2::response_mut::ResponseMut<
    crate::LocalService,
    [CustomPayloadMarker],
    CustomHeaderMarker,
>;

pub(crate) enum ResponseMutType {
    Ipc(Option<IpcResponseMut>),
    Local(Option<LocalResponseMut>),
}

#[pyclass]
pub struct ResponseMut {
    pub(crate) value: Parc<ResponseMutType>,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl ResponseMut {
    #[getter]
    pub fn __response_payload_type_details(&self) -> Option<Py<PyAny>> {
        self.response_payload_type_details.clone().value
    }

    #[getter]
    pub fn __response_header_type_details(&self) -> Option<Py<PyAny>> {
        self.response_header_type_details.clone().value
    }

    #[getter]
    pub fn __slice_len(&self) -> usize {
        match &*self.value.lock() {
            ResponseMutType::Ipc(Some(v)) => v.payload().len(),
            ResponseMutType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "RequestMut::__slice_len()",
                "Accessing a released response mut."),
        }
    }

    #[getter]
    /// Returns a reference to the `ResponseHeader`.
    pub fn header(&self) -> ResponseHeader {
        match &*self.value.lock() {
            ResponseMutType::Ipc(Some(v)) => ResponseHeader(*v.header()),
            ResponseMutType::Local(Some(v)) => ResponseHeader(*v.header()),
            _ => fatal_panic!(from "ResponseMut::header()",
                    "Accessing a released response mut."),
        }
    }

    #[getter]
    /// Returns a pointer to the user header of the response.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            ResponseMutType::Ipc(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            ResponseMutType::Local(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            _ => fatal_panic!(from "ResponseMut::user_header_ptr()",
                    "Accessing a released response mut."),
        }
    }

    #[getter]
    /// Returns a pointer to the payload of the response.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            ResponseMutType::Ipc(Some(v)) => v.payload_mut().as_mut_ptr() as usize,
            ResponseMutType::Local(Some(v)) => v.payload_mut().as_mut_ptr() as usize,
            _ => fatal_panic!(from "ResponseMut::payload_ptr()",
                    "Accessing a released response mut."),
        }
    }

    /// Releases the `ResponseMut`.
    ///
    /// After this call the `ResponseMut` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            ResponseMutType::Ipc(ref mut v) => {
                v.take();
            }
            ResponseMutType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    /// Sends a `ResponseMut` to the corresponding `PendingResponse` of the
    /// `Client`.
    pub fn send(&self) -> PyResult<()> {
        match &mut *self.value.lock() {
            ResponseMutType::Ipc(ref mut v) => {
                let response = v.take().unwrap();
                Ok(response
                    .send()
                    .map_err(|e| SendError::new_err(format!("{e:?}")))?)
            }
            ResponseMutType::Local(ref mut v) => {
                let response = v.take().unwrap();
                Ok(response
                    .send()
                    .map_err(|e| SendError::new_err(format!("{e:?}")))?)
            }
        }
    }
}
