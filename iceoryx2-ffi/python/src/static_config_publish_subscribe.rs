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

use pyo3::prelude::*;

use crate::message_type_details::MessageTypeDetails;

#[pyclass]
/// The static configuration of an `MessagingPattern::PublishSubscribe` based `Service`. Contains
/// all parameters that do not change during the lifetime of a `Service`.
pub struct StaticConfigPublishSubscribe(
    pub(crate) iceoryx2::service::static_config::publish_subscribe::StaticConfig,
);

#[pymethods]
impl StaticConfigPublishSubscribe {
    #[getter]
    /// Returns the maximum supported amount of `Node`s that can open the `Service` in parallel.
    pub fn max_nodes(&self) -> usize {
        self.0.max_nodes()
    }

    #[getter]
    /// Returns the maximum supported amount of `Publisher` ports
    pub fn max_publishers(&self) -> usize {
        self.0.max_publishers()
    }

    #[getter]
    /// Returns the maximum supported amount of `Subscriber` ports
    pub fn max_subscribers(&self) -> usize {
        self.0.max_subscribers()
    }

    #[getter]
    /// Returns the maximum history size that can be requested on connect.
    pub fn history_size(&self) -> usize {
        self.0.history_size()
    }

    #[getter]
    /// Returns the maximum supported buffer size for `Subscriber` port
    pub fn subscriber_max_buffer_size(&self) -> usize {
        self.0.subscriber_max_buffer_size()
    }

    #[getter]
    /// Returns how many `Sample` a `Subscriber` port can borrow in parallel at most.
    pub fn subscriber_max_borrowed_samples(&self) -> usize {
        self.0.subscriber_max_borrowed_samples()
    }

    #[getter]
    /// Returns true if the `Service` safely overflows, otherwise false. Safe
    /// overflow means that the `Publisher` will recycle the oldest
    /// `Sample` from the `Subscriber` when its buffer
    /// is full.
    pub fn has_safe_overflow(&self) -> bool {
        self.0.has_safe_overflow()
    }

    #[getter]
    /// Returns the type details of the `Service`.
    pub fn message_type_details(&self) -> MessageTypeDetails {
        MessageTypeDetails(self.0.message_type_details().clone())
    }
}
