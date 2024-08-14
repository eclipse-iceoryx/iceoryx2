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

use iceoryx2::service::header::publish_subscribe::Header;
use iceoryx2_bb_elementary::static_assert::static_assert_ge;

use crate::iox2_unique_publisher_id_t;

// BEGIN types definition

/// Sample header used by [`MessagingPattern::PublishSubscribe`]
#[repr(C)]
#[repr(align(8))] // core::mem::align_of::<Header>()
pub struct iox2_publish_subscribe_header_t {
    value: [u8; core::mem::size_of::<Header>()],
}

impl iox2_publish_subscribe_header_t {
    pub(super) fn as_ref(&self) -> &Header {
        static_assert_ge::<{ core::mem::align_of::<Self>() }, { core::mem::align_of::<Header>() }>(
        );

        unsafe { core::mem::transmute(self.value.as_ptr()) }
    }

    pub(super) fn as_mut(&mut self) -> &mut Header {
        unsafe { core::mem::transmute(self.value.as_ptr()) }
    }
}

// END types definition

// BEGIN C API

/// Returns the unique publisher id of the source of the sample.
///
/// # Safety
///
/// * `handle` is valid, non-null and was initialized with
///    [`iox2_sample_header()`](crate::iox2_sample_header)
/// * `id` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_publisher_id(
    handle: *const iox2_publish_subscribe_header_t,
    id: *mut iox2_unique_publisher_id_t,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!id.is_null());

    *(*id).as_mut() = (*handle).as_ref().publisher_id()
}

// END C API
