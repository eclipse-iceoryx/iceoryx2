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

use crate::{listener::Listener, parc::Parc};

pub(crate) enum WaitSetType {
    Ipc(iceoryx2::waitset::WaitSet<crate::IpcService>),
    Local(iceoryx2::waitset::WaitSet<crate::LocalService>),
}

#[pyclass]
pub struct WaitSet(pub(crate) Parc<WaitSetType>);

impl WaitSet {
    pub fn attach_notification(&self, attachment: &Listener) {
        todo!()
    }
}
