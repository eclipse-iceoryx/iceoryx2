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
use iceoryx2_bb_container::string::*;
use iceoryx2_bb_container::vector::*;

#[derive(Debug, Default, PlacementDefault, ZeroCopySend)]
#[repr(C)]
pub struct FullName {
    pub first_name: StaticString<256>,
    pub last_name: StaticString<256>,
}

// We derive from PlacementDefault to allow in memory initialization
// without any copy. Avoids stack overflows when data type is larger than the
// available stack.
#[derive(Debug, Default, PlacementDefault, ZeroCopySend)]
// optional type name; if not set, `core::any::type_name::<ComplexType>()` is used
#[type_name("ComplexType")]
#[repr(C)]
pub struct ComplexType {
    pub address_book: StaticVec<FullName, 16384>,
    pub some_matrix: StaticVec<StaticVec<f64, 8>, 8>,
    pub some_value: u16,
    pub another_value: u32,
}
