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

#![allow(non_camel_case_types)]

use iceoryx2::port::port_identifiers::UniqueSubscriberId;
use iceoryx2_bb_elementary::static_assert::static_assert_ge;

// BEGIN types definition

/// The system-wide unique id of a [`iox2_subscriber_t`].
#[repr(C)]
#[repr(align(4))] // core::mem::align_of::<UniqueSubscriberId>()
pub struct iox2_unique_subscriber_id_t {
    value: [u8; 16], // core::mem::size_of::<UniqueSubscriberId>()
}

impl iox2_unique_subscriber_id_t {
    pub(super) fn as_ref(&self) -> &UniqueSubscriberId {
        static_assert_ge::<
            { core::mem::align_of::<Self>() },
            { core::mem::align_of::<UniqueSubscriberId>() },
        >();
        static_assert_ge::<
            { core::mem::size_of::<Self>() },
            { core::mem::size_of::<UniqueSubscriberId>() },
        >();

        unsafe { core::mem::transmute(self.value.as_ptr()) }
    }

    pub(super) fn as_mut(&mut self) -> &mut UniqueSubscriberId {
        unsafe { core::mem::transmute(self.value.as_ptr()) }
    }
}
// END types definition

// BEGIN C API

/// Checks two [`iox2_unique_subscriber_id_t`] for equality.
///
/// # Safety
///
/// * `lhs` - A valid [`iox2_unique_subscriber_id_t`]
/// * `rhs` - A valid [`iox2_unique_subscriber_id_t`]
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_subscriber_id_eq(
    lhs: *const iox2_unique_subscriber_id_t,
    rhs: *const iox2_unique_subscriber_id_t,
) -> bool {
    debug_assert!(!lhs.is_null());
    debug_assert!(!rhs.is_null());

    (*lhs).as_ref() == (*rhs).as_ref()
}

/// Checks the ordering of two [`iox2_unique_subscriber_id_t`].
///
/// # Safety
///
/// * `lhs` - A valid [`iox2_unique_subscriber_id_t`]
/// * `rhs` - A valid [`iox2_unique_subscriber_id_t`]
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_subscriber_id_less(
    lhs: *const iox2_unique_subscriber_id_t,
    rhs: *const iox2_unique_subscriber_id_t,
) -> bool {
    debug_assert!(!lhs.is_null());
    debug_assert!(!rhs.is_null());

    (*lhs).as_ref() < (*rhs).as_ref()
}

// END C API
