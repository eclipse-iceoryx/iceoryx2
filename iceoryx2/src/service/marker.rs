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

use core::fmt::Debug;
use core::marker::PhantomData;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

#[repr(C)]
#[derive(Debug, ZeroCopySend, Clone, Default)]
#[doc(hidden)]
pub struct CustomHeaderMarker {}

#[repr(C)]
#[derive(Debug, ZeroCopySend, Clone)]
#[doc(hidden)]
pub struct CustomPayloadMarker(u8);

#[repr(C)]
#[derive(ZeroCopySend, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub struct CustomKeyMarker(u8);

/// Marker Type to mark a payload as serialized via Flatbuffer.
#[repr(C)]
pub struct Flatbuffer<T> {
    _data: u8,
    _phantom: PhantomData<T>,
}

impl<T> Debug for Flatbuffer<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Flatbuffer<{}>", core::any::type_name::<T>())
    }
}

unsafe impl<T> ZeroCopySend for Flatbuffer<T> {
    unsafe fn type_name() -> &'static str {
        "iox2::Flatbuffer"
    }
}
