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

pub use iceoryx2_pal_concurrency_sync::atomic::fence;
pub use iceoryx2_pal_concurrency_sync::atomic::Ordering;

use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicBool(internal::AtomicBool);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU8(internal::AtomicU8);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU16(internal::AtomicU16);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU32(internal::AtomicU32);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicU64(internal::AtomicU64);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicUsize(internal::AtomicUsize);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI8(internal::AtomicI8);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI16(internal::AtomicI16);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI32(internal::AtomicI32);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicI64(internal::AtomicI64);

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AtomicIsize(internal::AtomicIsize);

macro_rules! Impl {
    ($type_name:ident, $base_type:ident) => {
        impl $type_name {
            #[inline]
            pub const fn new(v: $base_type) -> Self {
                Self(internal::$type_name::new(v))
            }
        }

        impl Deref for $type_name {
            type Target = internal::$type_name;
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

Impl!(AtomicBool, bool);
Impl!(AtomicU8, u8);
Impl!(AtomicU16, u16);
Impl!(AtomicU32, u32);
Impl!(AtomicU64, u64);
Impl!(AtomicUsize, usize);
Impl!(AtomicI8, i8);
Impl!(AtomicI16, i16);
Impl!(AtomicI32, i32);
Impl!(AtomicI64, i64);
Impl!(AtomicIsize, isize);

mod internal {
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicBool;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicI16;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicI32;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicI64;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicI8;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicIsize;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicU16;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicU32;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicU64;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicU8;
    pub use iceoryx2_pal_concurrency_sync::atomic::AtomicUsize;
}
