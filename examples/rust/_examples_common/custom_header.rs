// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#[derive(Default, Debug, ZeroCopySend)]
// optional type name; if not set, `core::any::type_name::<CustomHeader>()` is used
#[type_name("CustomHeader")]
#[repr(C)]
pub struct CustomHeader {
    pub version: i32,
    pub timestamp: u64,
}
