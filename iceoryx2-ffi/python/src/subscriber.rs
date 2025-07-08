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
    error::{ConnectionFailure, ReceiveError},
    parc::Parc,
    sample::{Sample, SampleType},
    type_storage::TypeStorage,
    unique_subscriber_id::UniqueSubscriberId,
};

pub(crate) enum SubscriberType {
    Ipc(
        Option<
            iceoryx2::port::subscriber::Subscriber<
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Option<
            iceoryx2::port::subscriber::Subscriber<
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Subscriber {
    pub(crate) value: Parc<SubscriberType>,
    pub(crate) payload_type_details: TypeStorage,
    pub(crate) user_header_type_details: TypeStorage,
}

#[pymethods]
impl Subscriber {
    #[getter]
    /// Returns the `UniqueSubscriberId` of the `Subscriber`
    pub fn id(&self) -> UniqueSubscriberId {
        match &*self.value.lock() {
            SubscriberType::Ipc(Some(v)) => UniqueSubscriberId(v.id()),
            SubscriberType::Local(Some(v)) => UniqueSubscriberId(v.id()),
            _ => fatal_panic!(from "Subscriber::id()",
                    "Accessing a released Subscriber."),
        }
    }

    #[getter]
    /// Returns the internal buffer size of the `Subscriber`.
    pub fn buffer_size(&self) -> usize {
        match &*self.value.lock() {
            SubscriberType::Ipc(Some(v)) => v.buffer_size(),
            SubscriberType::Local(Some(v)) => v.buffer_size(),
            _ => fatal_panic!(from "Subscriber::buffer_size()",
                    "Accessing a released Subscriber."),
        }
    }

    /// Returns true if the `Subscriber` has samples in the buffer that can be received with
    /// `Subscriber::receive`. Emits `ConnectionFailure` on error.
    pub fn has_samples(&self) -> PyResult<bool> {
        match &*self.value.lock() {
            SubscriberType::Ipc(Some(v)) => Ok(v
                .has_samples()
                .map_err(|e| ConnectionFailure::new_err(format!("{e:?}")))?),
            SubscriberType::Local(Some(v)) => Ok(v
                .has_samples()
                .map_err(|e| ConnectionFailure::new_err(format!("{e:?}")))?),
            _ => fatal_panic!(from "Subscriber::has_samples()",
                    "Accessing a released Subscriber."),
        }
    }

    /// Receives a `Sample` from `Publisher`. If no sample could be received `None` is returned.
    /// If a failure occurs `ReceiveError` is returned.
    pub fn receive(&self) -> PyResult<Option<Sample>> {
        match &*self.value.lock() {
            SubscriberType::Ipc(Some(v)) => Ok(unsafe {
                v.receive_custom_payload()
                    .map_err(|e| ReceiveError::new_err(format!("{e:?}")))?
                    .map(|s| Sample {
                        value: Parc::new(SampleType::Ipc(Some(s))),
                        payload_type_details: self.payload_type_details.clone(),
                        user_header_type_details: self.user_header_type_details.clone(),
                    })
            }),
            SubscriberType::Local(Some(v)) => Ok(unsafe {
                v.receive_custom_payload()
                    .map_err(|e| ReceiveError::new_err(format!("{e:?}")))?
                    .map(|s| Sample {
                        value: Parc::new(SampleType::Local(Some(s))),
                        payload_type_details: self.payload_type_details.clone(),
                        user_header_type_details: self.user_header_type_details.clone(),
                    })
            }),
            _ => fatal_panic!(from "Subscriber::receive()",
                    "Accessing a released Subscriber."),
        }
    }

    /// Releases the `Subscriber`.
    ///
    /// After this call the `Subscriber` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            SubscriberType::Ipc(ref mut v) => {
                v.take();
            }
            SubscriberType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
