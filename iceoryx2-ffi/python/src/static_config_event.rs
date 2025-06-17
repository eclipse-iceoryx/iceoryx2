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
pub struct StaticConfigEvent(pub(crate) iceoryx2::service::static_config::event::StaticConfig);

#[pymethods]
impl StaticConfigEvent {
    #[getter]
    pub fn deadline(&self) -> Option<Duration> {
        self.0.deadline().map(|v| Duration(v))
    }

    #[getter]
    pub fn max_nodes(&self) -> usize {
        self.0.max_nodes()
    }

    #[getter]
    pub fn max_notifiers(&self) -> usize {
        self.0.max_notifiers()
    }

    #[getter]
    pub fn max_listeners(&self) -> usize {
        self.0.max_listeners()
    }

    #[getter]
    pub fn event_id_max_value(&self) -> usize {
        self.0.event_id_max_value()
    }

    #[getter]
    pub fn notifier_created_event(&self) -> Option<EventId> {
        self.0.notifier_created_event().map(|v| EventId(v))
    }

    #[getter]
    pub fn notifier_dropped_event(&self) -> Option<EventId> {
        self.0.notifier_dropped_event().map(|v| EventId(v))
    }

    #[getter]
    pub fn notifier_dead_event(&self) -> Option<EventId> {
        self.0.notifier_dead_event().map(|v| EventId(v))
    }
}
