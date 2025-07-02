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

use core::any::Any;
use std::sync::Arc;

use pyo3::prelude::*;

use crate::{parc::Parc, waitset::WaitSetType};

pub(crate) struct StorageType<S: iceoryx2::prelude::Service + 'static> {
    pub(crate) guard: Option<iceoryx2::prelude::WaitSetGuard<'static, 'static, S>>,
    pub(crate) waitset: Parc<WaitSetType>,
    pub(crate) _attachment: Option<Arc<dyn Any>>,
}

unsafe impl<S: iceoryx2::prelude::Service + 'static> Send for StorageType<S> {}
unsafe impl<S: iceoryx2::prelude::Service + 'static> Sync for StorageType<S> {}

pub(crate) enum WaitSetGuardType {
    Ipc(StorageType<crate::IpcService>),
    Local(StorageType<crate::LocalService>),
}

#[pyclass]
/// Is returned when something is attached to the `WaitSet`. As soon as it goes out
/// of scope, the attachment is detached.
pub struct WaitSetGuard(pub(crate) WaitSetGuardType);

#[pymethods]
impl WaitSetGuard {
    /// Drops the `WaitSetGuard`. After this call the `WaitSetGuard` is no longer usable.
    pub fn delete(&mut self) {
        match self.0 {
            WaitSetGuardType::Ipc(ref mut v) => {
                // the waitset needs to be locked otherwise we encounter a race condition since the
                // waitset itself is not threadsafe and the WaitSetGuard works on a WaitSet
                // reference on drop
                let _guard = v.waitset.lock();
                v.guard.take();
            }
            WaitSetGuardType::Local(ref mut v) => {
                // the waitset needs to be locked otherwise we encounter a race condition since the
                // waitset itself is not threadsafe and the WaitSetGuard works on a WaitSet
                // reference on drop
                let _guard = v.waitset.lock();
                v.guard.take();
            }
        }
    }
}
