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

use iceoryx2_log::fatal_panic;
use pyo3::prelude::*;

use crate::{
    duration::Duration, error::ListenerWaitError, event_id::EventId,
    unique_listener_id::UniqueListenerId,
};

#[allow(clippy::large_enum_variant)] // used purely for python and there it will reside always in
// the heap
pub(crate) enum ListenerType {
    Ipc(Option<Arc<iceoryx2::port::listener::Listener<crate::IpcService>>>),
    Local(Option<Arc<iceoryx2::port::listener::Listener<crate::LocalService>>>),
}

#[pyclass]
/// Represents the receiving endpoint of an event based communication.
pub struct Listener(pub(crate) ListenerType);

#[pymethods]
impl Listener {
    #[getter]
    /// Returns the deadline of the corresponding `Service`.
    pub fn deadline(&self) -> Option<Duration> {
        match &self.0 {
            ListenerType::Ipc(Some(v)) => v.deadline().map(Duration),
            ListenerType::Local(Some(v)) => v.deadline().map(Duration),
            _ => fatal_panic!(from "Listener::deadline()",
                    "Accessing a released listener."),
        }
    }

    /// Non-blocking wait for new `EventId`s. Collects all `EventId`s that were received and
    /// calls the provided callback is with the `EventId` as input argument.
    /// On error it emits `ListenerWaitError`.
    pub fn try_wait_all(&self) -> PyResult<Vec<EventId>> {
        let mut event_ids = vec![];
        match &self.0 {
            ListenerType::Ipc(Some(v)) => v
                .try_wait_all(|e| event_ids.push(EventId(e)))
                .map_err(|e| ListenerWaitError::new_err(format!("{e:?}")))?,
            ListenerType::Local(Some(v)) => v
                .try_wait_all(|e| event_ids.push(EventId(e)))
                .map_err(|e| ListenerWaitError::new_err(format!("{e:?}")))?,
            _ => fatal_panic!(from "Listener::try_wait_all()",
                    "Accessing a released listener."),
        }

        Ok(event_ids)
    }

    /// Blocking wait for new `EventId`s until the provided timeout has passed. Unblocks as soon
    /// as an `EventId` was received and then collects all `EventId`s that were received and
    /// calls the provided callback is with the `EventId` as input argument.
    /// On error it emits `ListenerWaitError`.
    pub fn timed_wait_all(&self, timeout: &Duration, py: Python<'_>) -> PyResult<Vec<EventId>> {
        py.detach(move || {
            let mut event_ids = vec![];
            match &self.0 {
                ListenerType::Ipc(Some(v)) => v
                    .timed_wait_all(|e| event_ids.push(EventId(e)), timeout.0)
                    .map_err(|e| ListenerWaitError::new_err(format!("{e:?}")))?,
                ListenerType::Local(Some(v)) => v
                    .timed_wait_all(|e| event_ids.push(EventId(e)), timeout.0)
                    .map_err(|e| ListenerWaitError::new_err(format!("{e:?}")))?,
                _ => fatal_panic!(from "Listener::timed_wait_all()",
                        "Accessing a released listener."),
            }

            Ok(event_ids)
        })
    }

    /// Blocking wait for new `EventId`s. Unblocks as soon
    /// as an `EventId` was received and then collects all `EventId`s that were received and
    /// calls the provided callback is with the `EventId` as input argument.
    /// On error it emits `ListenerWaitError`.
    pub fn blocking_wait_all(&self, py: Python<'_>) -> PyResult<Vec<EventId>> {
        py.detach(move || {
            let mut event_ids = vec![];
            match &self.0 {
                ListenerType::Ipc(Some(v)) => {
                    v.blocking_wait_all(|e| event_ids.push(EventId(e)))
                        .map_err(|e| ListenerWaitError::new_err(format!("{e:?}")))?
                }
                ListenerType::Local(Some(v)) => v
                    .blocking_wait_all(|e| event_ids.push(EventId(e)))
                    .map_err(|e| ListenerWaitError::new_err(format!("{e:?}")))?,
                _ => fatal_panic!(from "Listener::blocking_wait_all()",
                    "Accessing a released listener."),
            }

            Ok(event_ids)
        })
    }

    #[getter]
    /// Returns the `UniqueListenerId` of the `Listener`
    pub fn id(&self) -> UniqueListenerId {
        match &self.0 {
            ListenerType::Ipc(Some(v)) => UniqueListenerId(v.id()),
            ListenerType::Local(Some(v)) => UniqueListenerId(v.id()),
            _ => fatal_panic!(from "Listener::id()",
                    "Accessing a released listener."),
        }
    }

    /// Releases the `Listener`.
    ///
    /// After this call the `Listener` is no longer usable!
    pub fn delete(&mut self) {
        match self.0 {
            ListenerType::Ipc(ref mut v) => {
                v.take();
            }
            ListenerType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
