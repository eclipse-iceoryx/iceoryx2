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

use std::sync::Arc;

use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::{
    error::LoanError,
    parc::Parc,
    sample_mut_uninit::{SampleMutUninit, SampleMutUninitType},
    unable_to_deliver_strategy::UnableToDeliverStrategy,
    unique_publisher_id::UniquePublisherId,
};

pub(crate) enum PublisherType {
    Ipc(
        Arc<
            iceoryx2::port::publisher::Publisher<
                crate::IpcService,
                [CustomPayloadMarker],
                CustomHeaderMarker,
            >,
        >,
    ),
    Local(
        Arc<
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
pub struct Publisher(pub(crate) PublisherType);

#[pymethods]
impl Publisher {
    #[getter]
    /// Returns the `UniquePublisherId` of the `Publisher`
    pub fn id(&self) -> UniquePublisherId {
        match &self.0 {
            PublisherType::Ipc(v) => UniquePublisherId(v.id()),
            PublisherType::Local(v) => UniquePublisherId(v.id()),
        }
    }

    /// Returns the strategy the `Publisher` follows when a `SampleMut` cannot be delivered
    /// since the `Subscriber`s buffer is full.
    pub fn unable_to_deliver_strategy(&self) -> UnableToDeliverStrategy {
        match &self.0 {
            PublisherType::Ipc(v) => v.unable_to_deliver_strategy().into(),
            PublisherType::Local(v) => v.unable_to_deliver_strategy().into(),
        }
    }

    /// Returns the maximum initial slice length configured for this `Publisher`.
    pub fn initial_max_slice_len(&self) -> usize {
        match &self.0 {
            PublisherType::Ipc(v) => v.initial_max_slice_len(),
            PublisherType::Local(v) => v.initial_max_slice_len(),
        }
    }

    /// Loans/allocates a `SampleMutUninit` from the underlying data segment of the `Publisher`.
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns `LoanError` describing the failure.
    pub fn loan_slice_uninit(&self, number_of_elements: usize) -> PyResult<SampleMutUninit> {
        match &self.0 {
            PublisherType::Ipc(v) => {
                let sample = unsafe {
                    v.loan_custom_payload(number_of_elements)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                };
                Ok(SampleMutUninit(Parc::new(SampleMutUninitType::Ipc(Some(
                    sample,
                )))))
            }
            PublisherType::Local(v) => {
                let sample = unsafe {
                    v.loan_custom_payload(number_of_elements)
                        .map_err(|e| LoanError::new_err(format!("{e:?}")))?
                };
                Ok(SampleMutUninit(Parc::new(SampleMutUninitType::Local(
                    Some(sample),
                ))))
            }
        }
    }
}
