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

use core::marker::PhantomData;

use pyo3::prelude::*;

struct StorageType<S: iceoryx2::prelude::Service + 'static> {
    _data: PhantomData<S>, //    guard: iceoryx2::prelude::WaitSetGuard<'static, 'static, S>,
}

pub(crate) enum WaitSetGuardType {
    Ipc(StorageType<crate::IpcService>),
    Local(StorageType<crate::LocalService>),
}

#[pyclass]
pub struct WaitSetGuard(WaitSetGuardType);
