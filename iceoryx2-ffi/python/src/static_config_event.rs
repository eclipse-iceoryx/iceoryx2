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

use crate::{duration::Duration, event_id::EventId};

#[pyclass]
/// The static configuration of an `MessagingPattern::Event`
/// based service. Contains all parameters that do not change during the lifetime of a
/// `Service`.
pub struct StaticConfigEvent(pub(crate) iceoryx2::service::static_config::event::StaticConfig);

#[pymethods]
impl StaticConfigEvent {
    #[getter]
    /// Returns the deadline of the service. If no new notification is signaled from any
    /// `Notifier` after the given deadline, it is rated as an error and all `Listener`s that are
    /// attached to a `WaitSet` are woken up and notified about the missed
    pub fn deadline(&self) -> Option<Duration> {
        self.0.deadline().map(Duration)
    }

    #[getter]
    /// Returns the maximum supported amount of `Node`s that can open the `Service` in parallel.
    pub fn max_nodes(&self) -> usize {
        self.0.max_nodes()
    }

    #[getter]
    /// Returns the maximum supported amount of `Notifier` ports
    pub fn max_notifiers(&self) -> usize {
        self.0.max_notifiers()
    }

    #[getter]
    /// Returns the maximum supported amount of `Listener` ports
    pub fn max_listeners(&self) -> usize {
        self.0.max_listeners()
    }

    #[getter]
    /// Returns the largest `EventId` that is supported by the service
    pub fn event_id_max_value(&self) -> usize {
        self.0.event_id_max_value()
    }

    #[getter]
    /// Returns the emitted `EventId` when a new notifier is created.
    pub fn notifier_created_event(&self) -> Option<EventId> {
        self.0.notifier_created_event().map(EventId)
    }

    #[getter]
    /// Returns the emitted `EventId` when a notifier is dropped.
    pub fn notifier_dropped_event(&self) -> Option<EventId> {
        self.0.notifier_dropped_event().map(EventId)
    }

    #[getter]
    /// Returns the emitted `EventId` when a notifier is identified as dead.
    pub fn notifier_dead_event(&self) -> Option<EventId> {
        self.0.notifier_dead_event().map(EventId)
    }
}
