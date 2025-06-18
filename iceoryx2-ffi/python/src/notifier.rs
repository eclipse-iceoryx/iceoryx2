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

use iceoryx2::prelude::{ipc, local};
use pyo3::prelude::*;

use crate::{
    duration::Duration, error::NotifierNotifyError, event_id::EventId,
    unique_notifier_id::UniqueNotifierId,
};

pub(crate) enum NotifierType {
    Ipc(iceoryx2::port::notifier::Notifier<ipc::Service>),
    Local(iceoryx2::port::notifier::Notifier<local::Service>),
}

#[pyclass]
pub struct Notifier(pub(crate) Mutex<NotifierType>);

#[pymethods]
impl Notifier {
    #[getter]
    pub fn id(&self) -> UniqueNotifierId {
        match &*self.0.lock().unwrap() {
            NotifierType::Ipc(v) => UniqueNotifierId(v.id()),
            NotifierType::Local(v) => UniqueNotifierId(v.id()),
        }
    }

    #[getter]
    pub fn deadline(&self) -> Option<Duration> {
        match &*self.0.lock().unwrap() {
            NotifierType::Ipc(v) => v.deadline().map(Duration),
            NotifierType::Local(v) => v.deadline().map(Duration),
        }
    }

    pub fn notify(&self) -> PyResult<usize> {
        match &*self.0.lock().unwrap() {
            NotifierType::Ipc(v) => Ok(v
                .notify()
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
            NotifierType::Local(v) => Ok(v
                .notify()
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
        }
    }

    pub fn notify_with_custom_event_id(&self, event_id: &EventId) -> PyResult<usize> {
        match &*self.0.lock().unwrap() {
            NotifierType::Ipc(v) => Ok(v
                .notify_with_custom_event_id(event_id.0)
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
            NotifierType::Local(v) => Ok(v
                .notify_with_custom_event_id(event_id.0)
                .map_err(|e| NotifierNotifyError::new_err(format!("{e:?}")))?),
        }
    }
}
