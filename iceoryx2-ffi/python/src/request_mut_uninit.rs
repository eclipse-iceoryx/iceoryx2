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
use iceoryx2_bb_log::fatal_panic;
use pyo3::prelude::*;

use crate::{
    parc::Parc,
    request_header::RequestHeader,
    request_mut::{RequestMut, RequestMutType},
    type_storage::TypeStorage,
};

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

#[pymethods]
impl RequestMutUninit {
    #[getter]
    pub fn __request_payload_type_details(&self) -> Option<Py<PyAny>> {
        self.request_payload_type_details.clone().value
    }

    #[getter]
    pub fn __request_header_type_details(&self) -> Option<Py<PyAny>> {
        self.request_header_type_details.clone().value
    }

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
            RequestMutUninitType::Ipc(Some(v)) => v.payload().len(),
            RequestMutUninitType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "RequestMutUninit::__slice_len()",
                "Accessing a released request."),
        }
    }

    #[getter]
    /// Returns the iceoryx2 internal `RequestHeader`
    pub fn header(&self) -> RequestHeader {
        match &*self.value.lock() {
            RequestMutUninitType::Ipc(Some(v)) => RequestHeader(*v.header()),
            RequestMutUninitType::Local(Some(v)) => RequestHeader(*v.header()),
            _ => fatal_panic!(from "RequestMutUninit::header()",
                "Accessing a released request."),
        }
    }

    #[getter]
    /// Returns a pointer to the user defined request header.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            RequestMutUninitType::Ipc(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            RequestMutUninitType::Local(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            _ => fatal_panic!(from "RequestMutUninit::user_header_ptr()",
                "Accessing a released request."),
        }
    }

    #[getter]
    /// Returns a pointer to the user defined request payload.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            RequestMutUninitType::Ipc(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            RequestMutUninitType::Local(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            _ => fatal_panic!(from "RequestMutUninit::payload_ptr()",
                "Accessing a released request."),
        }
    }

    /// Releases the `RequestMutUninit`.
    ///
    /// After this call the `RequestMutUninit` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            RequestMutUninitType::Ipc(v) => {
                v.take();
            }
            RequestMutUninitType::Local(v) => {
                v.take();
            }
        }
    }

    /// When the payload is manually populated by using
    /// `RequestMutUninit::payload_ptr()`, then this function can be used
    /// to convert it into the initialized `RequestMut` version.
    pub fn assume_init(&self) -> RequestMut {
        match &mut *self.value.lock() {
            RequestMutUninitType::Ipc(ref mut v) => {
                let request = v.take().unwrap();
                RequestMut {
                    value: Parc::new(RequestMutType::Ipc(Some(unsafe { request.assume_init() }))),
                    request_header_type_details: self.request_header_type_details.clone(),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                }
            }
            RequestMutUninitType::Local(ref mut v) => {
                let request = v.take().unwrap();
                RequestMut {
                    value: Parc::new(RequestMutType::Local(Some(unsafe {
                        request.assume_init()
                    }))),
                    request_header_type_details: self.request_header_type_details.clone(),
                    request_payload_type_details: self.request_payload_type_details.clone(),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                }
            }
        }
    }
}
