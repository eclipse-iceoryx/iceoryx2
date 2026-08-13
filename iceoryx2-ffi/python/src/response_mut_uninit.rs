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

use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2_log::fatal_panic;
use pyo3::prelude::*;

use crate::{
    error::AllocationGrowError,
    parc::Parc,
    response_header::ResponseHeader,
    response_mut::{ResponseMut, ResponseMutType},
    type_storage::TypeStorage,
};

type IpcResponseMutUninit = iceoryx2::response_mut_uninit::ResponseMutUninit<
    crate::IpcService,
    [MaybeUninit<CustomPayloadMarker>],
    CustomHeaderMarker,
>;
type LocalResponseMutUninit = iceoryx2::response_mut_uninit::ResponseMutUninit<
    crate::LocalService,
    [MaybeUninit<CustomPayloadMarker>],
    CustomHeaderMarker,
>;

pub(crate) enum ResponseMutUninitType {
    Ipc(Option<IpcResponseMutUninit>),
    Local(Option<LocalResponseMutUninit>),
}

#[pyclass]
/// Acquired by a `ActiveRequest` with
///  * `ActiveRequest::loan_uninit()`
///
/// It stores the payload of the response that will be sent to the corresponding
/// `PendingResponse` of the `Client`.
///
/// If the `ResponseMutUninit` is not sent it will reelase the loaned memory when going out of
/// scope.
pub struct ResponseMutUninit {
    pub(crate) value: Parc<ResponseMutUninitType>,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl ResponseMutUninit {
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
            ResponseMutUninitType::Ipc(Some(v)) => v.payload().len(),
            ResponseMutUninitType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "ResponseMutUninit::__slice_len()",
                "Accessing a released response."),
        }
    }

    #[getter]
    pub fn __available_payload_memory(&self) -> usize {
        match &*self.value.lock() {
            ResponseMutUninitType::Ipc(Some(v)) => v.__internal_available_payload_memory(),
            ResponseMutUninitType::Local(Some(v)) => v.__internal_available_payload_memory(),
            _ => fatal_panic!(from "ResponseMutUninit::__available_payload_memory()",
                              "Accessing a released response."),
        }
    }

    pub fn __assume_init_flatbuffer(
        &self,
        buffer_ptr: usize,
        buffer_len: usize,
        payload_offset: usize,
    ) -> PyResult<ResponseMut> {
        let payload_ptr = (buffer_ptr + payload_offset) as *const u8;
        let payload_len = buffer_len - payload_offset;
        match &mut *self.value.lock() {
            ResponseMutUninitType::Ipc(v) => {
                let mut response = v.take().unwrap();
                let mut memory_buffer = response.__internal_create_resizable_memory();
                if memory_buffer.len() < buffer_len {
                    memory_buffer
                        .grow_downwards_with_size(buffer_len, 0)
                        .map_err(|e| AllocationGrowError::new_err(format!("{e:?}")))?
                }
                unsafe {
                    core::ptr::copy(
                        payload_ptr,
                        memory_buffer.as_mut_ptr().add(payload_offset),
                        payload_len,
                    )
                };

                response.__internal_finish_serialized(unsafe {
                    memory_buffer.as_ptr().add(payload_offset)
                });
                Ok(ResponseMut {
                    value: Parc::new(ResponseMutType::Ipc(Some(unsafe {
                        response.assume_init()
                    }))),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                })
            }
            ResponseMutUninitType::Local(v) => {
                let mut response = v.take().unwrap();
                let mut memory_buffer = response.__internal_create_resizable_memory();
                if memory_buffer.len() < buffer_len {
                    memory_buffer
                        .grow_downwards_with_size(buffer_len, 0)
                        .map_err(|e| AllocationGrowError::new_err(format!("{e:?}")))?
                }
                unsafe {
                    core::ptr::copy(
                        payload_ptr,
                        memory_buffer.as_mut_ptr().add(payload_offset),
                        payload_len,
                    )
                };

                response.__internal_finish_serialized(unsafe {
                    memory_buffer.as_ptr().add(payload_offset)
                });
                Ok(ResponseMut {
                    value: Parc::new(ResponseMutType::Local(Some(unsafe {
                        response.assume_init()
                    }))),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                })
            }
        }
    }

    pub fn __assume_init(&self) -> ResponseMut {
        match &mut *self.value.lock() {
            ResponseMutUninitType::Ipc(v) => {
                let response = v.take().unwrap();
                ResponseMut {
                    value: Parc::new(ResponseMutType::Ipc(Some(unsafe {
                        response.assume_init()
                    }))),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                }
            }
            ResponseMutUninitType::Local(v) => {
                let response = v.take().unwrap();
                ResponseMut {
                    value: Parc::new(ResponseMutType::Local(Some(unsafe {
                        response.assume_init()
                    }))),
                    response_header_type_details: self.response_header_type_details.clone(),
                    response_payload_type_details: self.response_payload_type_details.clone(),
                }
            }
        }
    }

    #[getter]
    /// Returns a reference to the `ResponseHeader`.
    pub fn header(&self) -> ResponseHeader {
        match &*self.value.lock() {
            ResponseMutUninitType::Ipc(Some(v)) => ResponseHeader(*v.header()),
            ResponseMutUninitType::Local(Some(v)) => ResponseHeader(*v.header()),
            _ => fatal_panic!(from "ResponseMutUninit::header()",
                    "Accessing a released response mut uninit."),
        }
    }

    #[getter]
    /// Returns a pointer to the user header of the response.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            ResponseMutUninitType::Ipc(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            ResponseMutUninitType::Local(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            _ => fatal_panic!(from "ResponseMutUninit::user_header_ptr()",
                    "Accessing a released response mut uninit."),
        }
    }

    #[getter]
    /// Returns a pointer to the payload of the response.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            ResponseMutUninitType::Ipc(Some(v)) => v.payload_mut().as_mut_ptr() as usize,
            ResponseMutUninitType::Local(Some(v)) => v.payload_mut().as_mut_ptr() as usize,
            _ => fatal_panic!(from "ResponseMutUninit::payload_ptr()",
                    "Accessing a released response mut uninit."),
        }
    }

    /// Releases the `ResponseMutUninit`.
    ///
    /// After this call the `ResponseMutUninit` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            ResponseMutUninitType::Ipc(v) => {
                v.take();
            }
            ResponseMutUninitType::Local(v) => {
                v.take();
            }
        }
    }
}
