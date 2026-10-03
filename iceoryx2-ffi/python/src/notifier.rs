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

use std::sync::Mutex;

use iceoryx2_log::fatal_panic;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::{
    duration::Duration,
    error::NotifierNotifyError,
    event_id::EventId,
    notifier_access::{AccessError, AccessKind, NotifierAccess, Permit},
    notifier_callback::{self, ListenerKey},
    port_name::PortName,
    unique_notifier_id::UniqueNotifierId,
};

#[allow(clippy::large_enum_variant)] // allowed since they are the same port type based on a different service variant
pub(crate) enum NotifierType {
    Ipc(Option<iceoryx2::port::notifier::Notifier<crate::IpcService>>),
    Local(Option<iceoryx2::port::notifier::Notifier<crate::LocalService>>),
}

#[pyclass]
/// Represents the sending endpoint of an event based communication.
///
/// Ordinary concurrent calls wait for each other. During `for_each_listener`,
/// all other methods and properties on this notifier raise `RuntimeError`, even
/// from another thread. Use the callback's `Monofier` to notify its listener.
/// Re-entry from an ordinary method (for example via logging) also raises
/// `RuntimeError` instead of waiting on itself.
pub struct Notifier {
    value: Mutex<NotifierType>,
    access: NotifierAccess,
}

impl Notifier {
    pub(crate) fn new(value: NotifierType) -> Self {
        Self {
            value: Mutex::new(value),
            access: NotifierAccess::default(),
        }
    }

    fn acquire(&self, py: Python<'_>, kind: AccessKind) -> PyResult<Permit<'_>> {
        py.detach(|| self.access.acquire(kind)).map_err(|error| {
            PyRuntimeError::new_err(match error {
                AccessError::TraversalActive => "Notifier is being traversed",
                AccessError::Reentrant => "Notifier operation cannot re-enter itself",
                AccessError::Poisoned => "Notifier admission lock is poisoned",
            })
        })
    }
}

#[pymethods]
impl Notifier {
    #[getter]
    /// Returns the `UniqueNotifierId` of the `Notifier`
    pub fn id(&self, py: Python<'_>) -> PyResult<UniqueNotifierId> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &*self.value.lock().unwrap() {
            NotifierType::Ipc(Some(v)) => Ok(UniqueNotifierId(v.id())),
            NotifierType::Local(Some(v)) => Ok(UniqueNotifierId(v.id())),
            _ => fatal_panic!(from "Notifier::id()",
                "Accessing a released notifier."),
        }
    }

    #[getter]
    /// Returns the `PortName` of the `Notifier`
    pub fn name(&self, py: Python<'_>) -> PyResult<PortName> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &*self.value.lock().unwrap() {
            NotifierType::Ipc(Some(v)) => Ok(PortName(*v.name())),
            NotifierType::Local(Some(v)) => Ok(PortName(*v.name())),
            _ => fatal_panic!(from "Notifier::name()",
                              "Accessing a released notifier."),
        }
    }

    #[getter]
    /// Returns the deadline of the corresponding `Service`.
    pub fn deadline(&self, py: Python<'_>) -> PyResult<Option<Duration>> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &*self.value.lock().unwrap() {
            NotifierType::Ipc(Some(v)) => Ok(v.deadline().map(Duration)),
            NotifierType::Local(Some(v)) => Ok(v.deadline().map(Duration)),
            _ => fatal_panic!(from "Notifier::deadline()",
                "Accessing a released notifier."),
        }
    }

    /// Notifies all `Listener` connected to the service with the default
    /// event id provided on creation.
    /// Returns on success the number of `Listener` ports that were notified otherwise it emits
    /// `NotifierNotifyError`.
    pub fn notify(&self, py: Python<'_>) -> PyResult<usize> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &*self.value.lock().unwrap() {
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
    /// Returns on success the number of `Listener` ports that were notified otherwise it returns
    /// `NotifierNotifyError`.
    pub fn notify_with_custom_event_id(
        &self,
        py: Python<'_>,
        event_id: &EventId,
    ) -> PyResult<usize> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &*self.value.lock().unwrap() {
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

    /// Notifies the listener corresponding to a saved `ListenerKey`.
    /// A deleted listener, reused slot, or key from another service emits
    /// `NotifierNotifyError`. A key does not keep its listener alive.
    pub fn notify_single_listener(&self, py: Python<'_>, key: &ListenerKey) -> PyResult<()> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &*self.value.lock().unwrap() {
            NotifierType::Ipc(Some(v)) => v.notify_single_listener(&key.0),
            NotifierType::Local(Some(v)) => v.notify_single_listener(&key.0),
            _ => fatal_panic!(from "Notifier::notify_single_listener()", "Accessing a released notifier."),
        }.map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))
    }

    /// Notifies a listener identified by a saved key with a custom `EventId`.
    /// Emits `NotifierNotifyError` on failure, preserving EventId validation
    /// precedence over invalid-key errors.
    pub fn notify_single_listener_with_custom_event_id(
        &self,
        py: Python<'_>,
        key: &ListenerKey,
        event_id: &EventId,
    ) -> PyResult<()> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &*self.value.lock().unwrap() {
            NotifierType::Ipc(Some(v)) => v.notify_single_listener_with_custom_event_id(&key.0, event_id.0),
            NotifierType::Local(Some(v)) => v.notify_single_listener_with_custom_event_id(&key.0, event_id.0),
            _ => fatal_panic!(from "Notifier::notify_single_listener_with_custom_event_id()", "Accessing a released notifier."),
        }.map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))
    }

    /// Calls ``callback(monofier, details)`` for each connected listener.
    /// Return `CallbackProgression.Continue` to continue or `CallbackProgression.Stop` to finish.
    /// Other return values raise `TypeError`. Callback exceptions propagate
    /// unchanged after cleanup; notifications already sent are not rolled back.
    ///
    /// Each Monofier is valid only on the callback's thread until that callback
    /// returns. Saved Monofiers never become valid in a later callback.
    /// Keys and details are independent copies and may be retained.
    ///
    /// Until traversal finishes, all other methods and properties on this
    /// notifier (including `delete`) raise `RuntimeError`. Different notifiers
    /// remain usable. Ordinary concurrent operations outside traversal wait.
    pub fn for_each_listener(&self, py: Python<'_>, callback: &Bound<'_, PyAny>) -> PyResult<()> {
        let _permit = self.acquire(py, AccessKind::Traversal)?;
        match &*self.value.lock().unwrap() {
            NotifierType::Ipc(Some(v)) => notifier_callback::for_each_listener(py, v, callback),
            NotifierType::Local(Some(v)) => notifier_callback::for_each_listener(py, v, callback),
            _ => {
                fatal_panic!(from "Notifier::for_each_listener()", "Accessing a released notifier.")
            }
        }
    }

    /// Releases the `Notifier`.
    ///
    /// After this call the `Notifier` is no longer usable!
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let _permit = self.acquire(py, AccessKind::Ordinary)?;
        match &mut *self.value.lock().unwrap() {
            NotifierType::Ipc(v) => {
                v.take();
            }
            NotifierType::Local(v) => {
                v.take();
            }
        }
        Ok(())
    }
}
