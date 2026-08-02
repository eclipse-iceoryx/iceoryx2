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
    header_publish_subscribe::HeaderPublishSubscribe,
    parc::Parc,
    sample_mut::{SampleMut, SampleMutType},
    type_storage::TypeStorage,
};

pub(crate) enum SampleMutUninitType {
    Ipc(
        Option<
            iceoryx2::sample_mut_uninit::SampleMutUninit<
                crate::IpcService,
                [MaybeUninit<CustomPayloadMarker>],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Option<
            iceoryx2::sample_mut_uninit::SampleMutUninit<
                crate::LocalService,
                [MaybeUninit<CustomPayloadMarker>],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
/// Acquired by a `Publisher` via
///  * `Publisher::loan_uninit()`
///
/// It stores the payload that will be sent
/// to all connected `Subscriber`s. If the `SampleMut` is not sent
/// it will release the loaned memory when going out of scope.
pub struct SampleMutUninit {
    pub(crate) value: Parc<SampleMutUninitType>,
    pub(crate) payload_type_details: TypeStorage,
    pub(crate) user_header_type_details: TypeStorage,
}

#[pymethods]
impl SampleMutUninit {
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
            SampleMutUninitType::Ipc(Some(v)) => v.payload().len(),
            SampleMutUninitType::Local(Some(v)) => v.payload().len(),
            _ => fatal_panic!(from "Sample::__slice_len()",
                "Accessing a released sample."),
        }
    }

    pub fn __assume_init_flatbuffer(
        &self,
        buffer_ptr: usize,
        buffer_len: usize,
        payload_offset: usize,
    ) -> PyResult<SampleMut> {
        let payload_ptr = (buffer_ptr + payload_offset) as *const u8;
        let payload_len = buffer_len - payload_offset;
        match &mut *self.value.lock() {
            SampleMutUninitType::Ipc(v) => {
                let mut sample = v.take().unwrap();
                let mut memory_buffer = sample.__internal_create_resizable_memory_builder();
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

                sample.__internal_finish_serialized(unsafe {
                    memory_buffer.as_ptr().add(payload_offset)
                });
                Ok(SampleMut {
                    value: Parc::new(SampleMutType::Ipc(Some(unsafe { sample.assume_init() }))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                })
            }
            SampleMutUninitType::Local(v) => {
                let mut sample = v.take().unwrap();
                let mut memory_buffer = sample.__internal_create_resizable_memory_builder();
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

                sample.__internal_finish_serialized(unsafe {
                    memory_buffer.as_ptr().add(payload_offset)
                });
                Ok(SampleMut {
                    value: Parc::new(SampleMutType::Local(Some(unsafe { sample.assume_init() }))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                })
            }
        }
    }

    pub fn __assume_init(&self) -> SampleMut {
        match &mut *self.value.lock() {
            SampleMutUninitType::Ipc(v) => {
                let sample = v.take().unwrap();
                SampleMut {
                    value: Parc::new(SampleMutType::Ipc(Some(unsafe { sample.assume_init() }))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                }
            }
            SampleMutUninitType::Local(v) => {
                let sample = v.take().unwrap();
                SampleMut {
                    value: Parc::new(SampleMutType::Local(Some(unsafe { sample.assume_init() }))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                }
            }
        }
    }

    #[getter]
    /// Returns the `HeaderPublishSubscribe` of the `Sample`.
    pub fn header(&self) -> HeaderPublishSubscribe {
        match &*self.value.lock() {
            SampleMutUninitType::Ipc(Some(v)) => HeaderPublishSubscribe(*v.header()),
            SampleMutUninitType::Local(Some(v)) => HeaderPublishSubscribe(*v.header()),
            _ => fatal_panic!(from "SampleMutUninit::header()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the user header.
    pub fn user_header_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            SampleMutUninitType::Ipc(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            SampleMutUninitType::Local(Some(v)) => {
                (v.user_header_mut() as *mut CustomHeaderMarker) as usize
            }
            _ => fatal_panic!(from "SampleMutUninit::user_header_ptr()",
                "Accessing a released sample."),
        }
    }

    #[getter]
    /// Returns a pointer to the payload.
    pub fn payload_ptr(&self) -> usize {
        match &mut *self.value.lock() {
            SampleMutUninitType::Ipc(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            SampleMutUninitType::Local(Some(v)) => (v.payload_mut().as_mut_ptr()) as usize,
            _ => fatal_panic!(from "SampleMutUninit::payload_ptr()",
                "Accessing a released sample."),
        }
    }

    /// Releases the `SampleMutUninit`.
    ///
    /// After this call the `SampleMutUninit` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            SampleMutUninitType::Ipc(v) => {
                v.take();
            }
            SampleMutUninitType::Local(v) => {
                v.take();
            }
        }
    }
}
