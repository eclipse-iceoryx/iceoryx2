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

macro_rules! zero_copy_send_atomic {
    ($type_name:ident, $base_type:ident) => {
        paste::paste! {
            use iceoryx2_pal_concurrency_sync::atomic::$type_name as [<Internal $type_name>];

            #[derive(Debug, Default)]
            #[repr(transparent)]
            pub struct $type_name([<Internal $type_name>]);

            impl $type_name {
                #[inline]
                pub const fn new(v: $base_type) -> Self {
                    Self([<Internal $type_name>]::new(v))
                }
            }

            impl Deref for $type_name {
                type Target = [<Internal $type_name>];
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
        }
    };
}

zero_copy_send_atomic!(AtomicBool, bool);
zero_copy_send_atomic!(AtomicU8, u8);
zero_copy_send_atomic!(AtomicU16, u16);
zero_copy_send_atomic!(AtomicU32, u32);
zero_copy_send_atomic!(AtomicU64, u64);
zero_copy_send_atomic!(AtomicUsize, usize);
zero_copy_send_atomic!(AtomicI8, i8);
zero_copy_send_atomic!(AtomicI16, i16);
zero_copy_send_atomic!(AtomicI32, i32);
zero_copy_send_atomic!(AtomicI64, i64);
zero_copy_send_atomic!(AtomicIsize, isize);
