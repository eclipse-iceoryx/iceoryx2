// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#[derive(Debug, Clone, Copy, ZeroCopySend)]
// optional type name; if not set, `core::any::type_name::<TransmissionData>()` is used
#[type_name("TransmissionData")]
#[repr(C)]
pub struct TransmissionData {
    pub x: i32,
    pub y: i32,
    pub funky: f64,
}
