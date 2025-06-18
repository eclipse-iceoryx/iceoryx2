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

use crate::{duration::Duration, event_id::EventId};

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
        todo!()
    }

    pub fn try_wait_one(&self) -> PyResult<Option<EventId>> {
        todo!()
    }

    pub fn timed_wait_one(&self, timeout: &Duration) -> PyResult<Option<EventId>> {
        todo!()
    }

    pub fn blocking_wait_one(&self) -> PyResult<Option<EventId>> {
        todo!()
    }

    pub fn try_wait_all(&self) -> PyResult<Vec<EventId>> {
        todo!()
    }

    pub fn timed_wait_all(&self, timeout: &Duration) -> PyResult<Vec<EventId>> {
        todo!()
    }

    pub fn blocking_wait_all(&self) -> PyResult<Vec<EventId>> {
        todo!()
    }

    #[getter]
    pub fn id(&self) -> usize {
        todo!()
    }
}
