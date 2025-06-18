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

use iceoryx2::prelude::{ipc, local};
use pyo3::prelude::*;

use crate::{
    duration::Duration, error::ListenerWaitError, event_id::EventId,
    unique_listener_id::UniqueListenerId,
};

#[allow(clippy::large_enum_variant)] // used purely for python and there it will reside always in
                                     // the heap
pub(crate) enum ListenerType {
    Ipc(iceoryx2::port::listener::Listener<ipc::Service>),
    Local(iceoryx2::port::listener::Listener<local::Service>),
}

#[pyclass]
pub struct Listener(pub(crate) ListenerType);

#[pymethods]
impl Listener {
    #[getter]
    pub fn deadline(&self) -> Option<Duration> {
        match &self.0 {
            ListenerType::Ipc(v) => v.deadline().map(Duration),
            ListenerType::Local(v) => v.deadline().map(Duration),
        }
    }

    pub fn try_wait_one(&self) -> PyResult<Option<EventId>> {
        match &self.0 {
            ListenerType::Ipc(v) => Ok(v
                .try_wait_one()
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?
                .map(EventId)),
            ListenerType::Local(v) => Ok(v
                .try_wait_one()
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?
                .map(EventId)),
        }
    }

    pub fn timed_wait_one(&self, timeout: &Duration) -> PyResult<Option<EventId>> {
        match &self.0 {
            ListenerType::Ipc(v) => Ok(v
                .timed_wait_one(timeout.0)
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?
                .map(EventId)),
            ListenerType::Local(v) => Ok(v
                .timed_wait_one(timeout.0)
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?
                .map(EventId)),
        }
    }

    pub fn blocking_wait_one(&self) -> PyResult<Option<EventId>> {
        match &self.0 {
            ListenerType::Ipc(v) => Ok(v
                .blocking_wait_one()
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?
                .map(EventId)),
            ListenerType::Local(v) => Ok(v
                .blocking_wait_one()
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?
                .map(EventId)),
        }
    }

    pub fn try_wait_all(&self) -> PyResult<Vec<EventId>> {
        let mut event_ids = vec![];
        match &self.0 {
            ListenerType::Ipc(v) => v
                .try_wait_all(|e| event_ids.push(EventId(e)))
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?,
            ListenerType::Local(v) => v
                .try_wait_all(|e| event_ids.push(EventId(e)))
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?,
        }

        Ok(event_ids)
    }

    pub fn timed_wait_all(&self, timeout: &Duration) -> PyResult<Vec<EventId>> {
        let mut event_ids = vec![];
        match &self.0 {
            ListenerType::Ipc(v) => v
                .timed_wait_all(|e| event_ids.push(EventId(e)), timeout.0)
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?,
            ListenerType::Local(v) => v
                .timed_wait_all(|e| event_ids.push(EventId(e)), timeout.0)
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?,
        }

        Ok(event_ids)
    }

    pub fn blocking_wait_all(&self) -> PyResult<Vec<EventId>> {
        let mut event_ids = vec![];
        match &self.0 {
            ListenerType::Ipc(v) => v
                .blocking_wait_all(|e| event_ids.push(EventId(e)))
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?,
            ListenerType::Local(v) => v
                .blocking_wait_all(|e| event_ids.push(EventId(e)))
                .map_err(|e| ListenerWaitError::new_err(format!("{:?}", e)))?,
        }

        Ok(event_ids)
    }

    #[getter]
    pub fn id(&self) -> UniqueListenerId {
        match &self.0 {
            ListenerType::Ipc(v) => UniqueListenerId(v.id()),
            ListenerType::Local(v) => UniqueListenerId(v.id()),
        }
    }
}
