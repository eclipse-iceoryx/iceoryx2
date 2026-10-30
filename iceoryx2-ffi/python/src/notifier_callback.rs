// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

//! Scoped bridge from a borrowed Rust Monofier to an arbitrarily retained Python object.
use std::{
    sync::Arc,
    thread::{self, ThreadId},
};

use crate::{
    error::NotifierNotifyError, event_id::EventId, port_name::PortName,
    unique_listener_id::UniqueListenerId, unique_node_id::UniqueNodeId,
};
use iceoryx2::{
    port::notifier::{
        Monofier as RustMonofier, Notifier as RustNotifier, NotifierNotifyError as RustError,
    },
    prelude::{CallbackProgression as RustProgression, EventId as RustEventId},
    service::{Service, dynamic_config::event::ListenerDetails as RustDetails},
};
use pyo3::{exceptions::PyRuntimeError, prelude::*};

#[pyclass(frozen)]
/// An independent key obtained from a Monofier. It may outlive the callback,
/// but does not keep the listener alive or make the key valid for another service.
pub struct ListenerKey(pub(crate) iceoryx2::port::notifier::ListenerKey);

#[pyclass(frozen)]
/// A snapshot of listener details, independent of the callback and listener lifetime.
pub struct ListenerDetails(RustDetails);

#[pymethods]
impl ListenerDetails {
    #[getter]
    fn listener_name(&self) -> PortName {
        PortName(self.0.listener_name)
    }
    #[getter]
    fn listener_id(&self) -> UniqueListenerId {
        UniqueListenerId(self.0.listener_id)
    }
    #[getter]
    fn node_id(&self) -> UniqueNodeId {
        UniqueNodeId(self.0.node_id)
    }
}

#[pyclass(eq, frozen)]
#[derive(PartialEq)]
/// Controls whether a listener traversal continues or stops.
pub enum CallbackProgression {
    Continue,
    Stop,
}

// The PAL has no AtomicPtr abstraction. This pointer belongs only to the Python
// runtime bridge and is never shared through iceoryx2 shared memory.
#[allow(clippy::disallowed_types)]
type AtomicPtr<T> = std::sync::atomic::AtomicPtr<T>;
#[allow(clippy::disallowed_types)]
type Ordering = std::sync::atomic::Ordering;

type Notify = unsafe fn(*mut (), Option<RustEventId>) -> Result<(), RustError>;

// All fields are private; only invoke_callback constructs an active state.
// Python owns the state, never the pointed-to Rust Monofier.
struct CallbackState {
    pointer: AtomicPtr<()>,
    owner: ThreadId,
    notify: Notify,
    key: iceoryx2::port::notifier::ListenerKey,
}

impl CallbackState {
    fn checked_pointer(&self) -> PyResult<*mut ()> {
        let pointer = self.pointer.load(Ordering::Acquire);
        if pointer.is_null() {
            return Err(PyRuntimeError::new_err("Monofier callback has ended"));
        }
        if thread::current().id() != self.owner {
            return Err(PyRuntimeError::new_err(
                "Monofier belongs to another thread",
            ));
        }
        Ok(pointer)
    }
}

struct Invalidate(Arc<CallbackState>);
impl Drop for Invalidate {
    fn drop(&mut self) {
        self.0
            .pointer
            .store(std::ptr::null_mut(), Ordering::Release);
    }
}

#[pyclass(frozen)]
/// Notifies the current listener without refreshing connections.
/// All methods raise RuntimeError outside the creating callback or thread.
pub struct Monofier {
    state: Arc<CallbackState>,
}

#[pymethods]
impl Monofier {
    /// Returns an independent key that can be retained after the callback.
    fn listener_key(&self) -> PyResult<ListenerKey> {
        self.state.checked_pointer()?;
        Ok(ListenerKey(self.state.key))
    }

    /// Notifies this listener using the notifier's default EventId.
    fn notify(&self) -> PyResult<()> {
        self.send(None)
    }

    /// Notifies this listener using a custom EventId.
    fn notify_with_custom_event_id(&self, event_id: &EventId) -> PyResult<()> {
        self.send(Some(event_id.0))
    }
}

impl Monofier {
    fn send(&self, event_id: Option<RustEventId>) -> PyResult<()> {
        let pointer = self.state.checked_pointer()?;
        // SAFETY: checked_pointer admits only the creating OS thread and a live
        // invocation. That thread cannot return from invoke_callback (and expire
        // its borrowed Rust Monofier) until this nested method returns. Other
        // threads can hold/drop the Arc but cannot dereference its pointer.
        // No method replaces the state or reactivates an expired invocation.
        unsafe { (self.state.notify)(pointer, event_id) }
            .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))
    }
}

unsafe fn notify_borrowed<S: Service>(
    pointer: *mut (),
    event_id: Option<RustEventId>,
) -> Result<(), RustError> {
    // SAFETY: only invoke_callback installs this function, with a pointer to the
    // matching RustMonofier<S>. Its reference is reconstructed for this call,
    // never stored as a 'static reference. The caller enforces scope and thread.
    let monofier = unsafe { &*pointer.cast::<RustMonofier<'_, S>>() };
    match event_id {
        Some(id) => monofier.notify_with_custom_event_id(id),
        None => monofier.notify(),
    }
}

fn invoke_callback<S: Service>(
    py: Python<'_>,
    callback: &Bound<'_, PyAny>,
    monofier: &RustMonofier<'_, S>,
    details: &RustDetails,
) -> PyResult<RustProgression> {
    let state = Arc::new(CallbackState {
        pointer: AtomicPtr::new((monofier as *const RustMonofier<'_, S>).cast_mut().cast()),
        owner: thread::current().id(),
        notify: notify_borrowed::<S>,
        key: monofier.listener_key(),
    });
    // RAII also covers Python allocation errors and Rust unwinding.
    let invalidate = Invalidate(state.clone());
    let py_monofier = Py::new(py, Monofier { state })?;
    let py_details = Py::new(py, ListenerDetails(*details))?;
    let result = callback.call1((&py_monofier, &py_details));
    // Expire before return-value extraction or dropping Python-owned values.
    drop(invalidate);
    let result = result?;
    let progression = result.extract::<PyRef<'_, CallbackProgression>>()?;
    Ok(match *progression {
        CallbackProgression::Continue => RustProgression::Continue,
        CallbackProgression::Stop => RustProgression::Stop,
    })
}

pub(crate) fn for_each_listener<S: Service>(
    py: Python<'_>,
    notifier: &RustNotifier<S>,
    callback: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let mut failure = None;
    notifier.for_each_listener(|monofier, details| {
        match invoke_callback(py, callback, monofier, details) {
            Ok(progression) => progression,
            Err(error) => {
                failure = Some(error);
                RustProgression::Stop
            }
        }
    });
    // The outer Notifier method releases its value lock and admission before returning this PyErr to Python.
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
