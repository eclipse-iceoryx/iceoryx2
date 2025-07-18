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

use iceoryx2_bb_log::fatal_panic;
use pyo3::prelude::*;

use crate::{
    duration::Duration, error::NotifierNotifyError, event_id::EventId,
    unique_notifier_id::UniqueNotifierId,
};

pub(crate) enum NotifierType {
    Ipc(Option<iceoryx2::port::notifier::Notifier<crate::IpcService>>),
    Local(Option<iceoryx2::port::notifier::Notifier<crate::LocalService>>),
}

#[pyclass]
/// Represents the sending endpoint of an event based communication.
pub struct Notifier(pub(crate) NotifierType);

#[pymethods]
impl Notifier {
    #[getter]
    /// Returns the `UniqueNotifierId` of the `Notifier`
    pub fn id(&self) -> UniqueNotifierId {
        match &self.0 {
            NotifierType::Ipc(Some(v)) => UniqueNotifierId(v.id()),
            NotifierType::Local(Some(v)) => UniqueNotifierId(v.id()),
            _ => fatal_panic!(from "Notifier::id()",
                "Accessing a released notifier."),
        }
    }

    #[getter]
    /// Returns the deadline of the corresponding `Service`.
    pub fn deadline(&self) -> Option<Duration> {
        match &self.0 {
            NotifierType::Ipc(Some(v)) => v.deadline().map(Duration),
            NotifierType::Local(Some(v)) => v.deadline().map(Duration),
            _ => fatal_panic!(from "Notifier::deadline()",
                "Accessing a released notifier."),
        }
    }

    /// Notifies all `Listener` connected to the service with the default
    /// event id provided on creation.
    /// Returns on success the number of `Listener`s that were notified otherwise it emits
    /// `NotifierNotifyError`.
    pub fn notify(&self) -> PyResult<usize> {
        match &self.0 {
            NotifierType::Ipc(Some(v)) => Ok(v
                .notify()
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
            NotifierType::Local(Some(v)) => Ok(v
                .notify()
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
            _ => fatal_panic!(from "Notifier::notify()",
                "Accessing a released notifier."),
        }
    }

    /// Notifies all `Listener` connected to the service with a custom `EventId`.
    /// Returns on success the number of `Listener`s that were notified otherwise it returns
    /// `NotifierNotifyError`.
    pub fn notify_with_custom_event_id(&self, event_id: &EventId) -> PyResult<usize> {
        match &self.0 {
            NotifierType::Ipc(Some(v)) => Ok(v
                .notify_with_custom_event_id(event_id.0)
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
            NotifierType::Local(Some(v)) => Ok(v
                .notify_with_custom_event_id(event_id.0)
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
            _ => fatal_panic!(from "Notifier::notify_with_custom_event_id()",
                "Accessing a released notifier."),
        }
    }

    /// Releases the `Notifier`.
    ///
    /// After this call the `Notifier` is no longer usable!
    pub fn delete(&mut self) {
        match self.0 {
            NotifierType::Ipc(ref mut v) => {
                v.take();
            }
            NotifierType::Local(ref mut v) => {
                v.take();
            }
        }
    }
}
