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

use core::ops::{Deref, DerefMut};

use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_pal_concurrency_sync::atomic::AtomicBool as InternalAtomicBool;
use iceoryx2_pal_concurrency_sync::atomic::AtomicI16 as InternalAtomicI16;
use iceoryx2_pal_concurrency_sync::atomic::AtomicI32 as InternalAtomicI32;
use iceoryx2_pal_concurrency_sync::atomic::AtomicI64 as InternalAtomicI64;
use iceoryx2_pal_concurrency_sync::atomic::AtomicI8 as InternalAtomicI8;
use iceoryx2_pal_concurrency_sync::atomic::AtomicIsize as InternalAtomicIsize;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU16 as InternalAtomicU16;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU32 as InternalAtomicU32;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU64 as InternalAtomicU64;
use iceoryx2_pal_concurrency_sync::atomic::AtomicU8 as InternalAtomicU8;
use iceoryx2_pal_concurrency_sync::atomic::AtomicUsize as InternalAtomicUsize;

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicBool(InternalAtomicBool);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU8(InternalAtomicU8);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU16(InternalAtomicU16);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU32(InternalAtomicU32);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU64(InternalAtomicU64);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicUsize(InternalAtomicUsize);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI8(InternalAtomicI8);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI16(InternalAtomicI16);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI32(InternalAtomicI32);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI64(InternalAtomicI64);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicIsize(InternalAtomicIsize);

macro_rules! zero_copy_send_atomic {
    ($type_name:ident, $wrapped_type_name:ident, $base_type:ident) => {
        impl $type_name {
            #[inline]
            pub const fn new(v: $base_type) -> Self {
                Self($wrapped_type_name::new(v))
            }
        }

        impl Deref for $type_name {
            type Target = $wrapped_type_name;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $type_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl PlacementDefault for $type_name {
            unsafe fn placement_default(ptr: *mut Self) {
                ptr.write(<$type_name>::default())
            }
        }

        unsafe impl ZeroCopySend for $type_name {}
    };
}

zero_copy_send_atomic!(AtomicBool, InternalAtomicBool, bool);
zero_copy_send_atomic!(AtomicU8, InternalAtomicU8, u8);
zero_copy_send_atomic!(AtomicU16, InternalAtomicU16, u16);
zero_copy_send_atomic!(AtomicU32, InternalAtomicU32, u32);
zero_copy_send_atomic!(AtomicU64, InternalAtomicU64, u64);
zero_copy_send_atomic!(AtomicUsize, InternalAtomicUsize, usize);
zero_copy_send_atomic!(AtomicI8, InternalAtomicI8, i8);
zero_copy_send_atomic!(AtomicI16, InternalAtomicI16, i16);
zero_copy_send_atomic!(AtomicI32, InternalAtomicI32, i32);
zero_copy_send_atomic!(AtomicI64, InternalAtomicI64, i64);
zero_copy_send_atomic!(AtomicIsize, InternalAtomicIsize, isize);
