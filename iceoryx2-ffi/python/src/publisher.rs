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
    error::LoanError,
    parc::Parc,
    sample_mut_uninit::{SampleMutUninit, SampleMutUninitType},
    type_storage::TypeStorage,
    unable_to_deliver_strategy::UnableToDeliverStrategy,
    unique_publisher_id::UniquePublisherId,
};

pub(crate) enum PublisherType {
    Ipc(
        Option<
            iceoryx2::port::publisher::Publisher<
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Option<
            iceoryx2::port::publisher::Publisher<
                crate::LocalService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Publisher {
    pub(crate) value: Parc<PublisherType>,
    pub(crate) payload_type_details: TypeStorage,
    pub(crate) user_header_type_details: TypeStorage,
}

#[pymethods]
impl Publisher {
    #[getter]
    pub fn __payload_type_details(&self) -> Option<Py<PyAny>> {
        self.payload_type_details.clone().value
    }

    #[getter]
    /// Returns the `UniquePublisherId` of the `Publisher`
    pub fn id(&self) -> UniquePublisherId {
        match &*self.value.lock() {
            PublisherType::Ipc(Some(v)) => UniquePublisherId(v.id()),
            PublisherType::Local(Some(v)) => UniquePublisherId(v.id()),
            _ => fatal_panic!(from "Publisher::id()",
                "Accessing a deleted publisher."),
        }
    }

    #[getter]
    /// Returns the strategy the `Publisher` follows when a `SampleMut` cannot be delivered
    /// since the `Subscriber`s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        match &*self.value.lock() {
            PublisherType::Ipc(Some(v)) => v.unable_to_deliver_strategy().into(),
            PublisherType::Local(Some(v)) => v.unable_to_deliver_strategy().into(),
            _ => fatal_panic!(from "Publisher::unable_to_deliver_strategy()",
                "Accessing a deleted publisher."),
        }
    }

    #[getter]
    /// Returns the maximum initial slice length configured for this `Publisher`.
    pub fn initial_max_slice_len(&self) -> usize {
        match &*self.value.lock() {
            PublisherType::Ipc(Some(v)) => v.initial_max_slice_len(),
            PublisherType::Local(Some(v)) => v.initial_max_slice_len(),
            _ => fatal_panic!(from "Publisher::initial_max_slice_len()",
                "Accessing a deleted publisher."),
        }
    }

    /// Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`.
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns `LoanError` describing the failure.
    pub fn __loan_uninit(&self) -> PyResult<SampleMutUninit> {
        self.__loan_slice_uninit(1)
    }

    /// Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`.
    /// The user has to initialize the payload before it can be sent.
    /// Fails when it is called for data types which are not a slice.
    ///
    /// On failure it returns `LoanError` describing the failure.
    pub fn __loan_slice_uninit(&self, number_of_elements: usize) -> PyResult<SampleMutUninit> {
        match &*self.value.lock() {
            PublisherType::Ipc(Some(v)) => {
                let sample = unsafe {
                    v.loan_custom_payload(number_of_elements)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                };
                Ok(SampleMutUninit {
                    value: Parc::new(SampleMutUninitType::Ipc(Some(sample))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                })
            }
            PublisherType::Local(Some(v)) => {
                let sample = unsafe {
                    v.loan_custom_payload(number_of_elements)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                };
                Ok(SampleMutUninit {
                    value: Parc::new(SampleMutUninitType::Local(Some(sample))),
                    payload_type_details: self.payload_type_details.clone(),
                    user_header_type_details: self.user_header_type_details.clone(),
                })
            }
            _ => fatal_panic!(from "Publisher::id()",
                "Accessing a deleted publisher."),
        }
    }

    /// Releases the `Publisher`.
    ///
    /// After this call the `Publisher` is no longer usable!
    pub fn delete(&mut self) {
        match &mut *self.value.lock() {
            PublisherType::Ipc(ref mut v) => {
                v.take();
            }
            PublisherType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
