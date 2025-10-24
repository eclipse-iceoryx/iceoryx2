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

use iceoryx2::prelude::*;

#[derive(Debug, Clone, Copy, ZeroCopySend, Hash, PartialEq, Eq)]
// optional type name; if not set, `core::any::type_name::<BlackboardKey>()` is used
#[type_name("BlackboardKey")]
#[repr(C)]
pub struct BlackboardKey {
    pub x: u32,
    pub y: i64,
    pub z: u16,
}
