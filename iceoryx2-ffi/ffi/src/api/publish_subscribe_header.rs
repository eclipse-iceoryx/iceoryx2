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
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::{
    api::AssertNonNullHandle, api::HandleToType, iox2_unique_publisher_id_h,
    iox2_unique_publisher_id_t,
};

// BEGIN types definition

/// Sample header used by `MessagingPattern::PublishSubscribe`
#[repr(C)]
#[repr(align(8))] // core::mem::align_of::<Option<Header>>()
pub struct iox2_publish_subscribe_header_storage_t {
    internal: [u8; 48], // core::mem::size_of::<Option<Header>>()
}

#[repr(C)]
#[iceoryx2_ffi(Header)]
pub struct iox2_publish_subscribe_header_t {
    pub value: iox2_publish_subscribe_header_storage_t,
    deleter: fn(*mut iox2_publish_subscribe_header_t),
}

impl iox2_publish_subscribe_header_t {
    pub(super) fn init(
        &mut self,
        header: Header,
        deleter: fn(*mut iox2_publish_subscribe_header_t),
    ) {
        self.value.init(header);
        self.deleter = deleter;
    }
}

pub struct iox2_publish_subscribe_header_h_t;
/// The owning handle for [`iox2_publish_subscribe_header_t`]. Passing the handle to an function transfers the ownership.
pub type iox2_publish_subscribe_header_h = *mut iox2_publish_subscribe_header_h_t;
/// The non-owning handle for [`iox2_publish_subscribe_header_t`]. Passing the handle to an function does not transfers the ownership.
pub type iox2_publish_subscribe_header_h_ref = *const iox2_publish_subscribe_header_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `publish_subscribe::Header`
pub type iox2_publish_subscribe_header_ptr = *const Header;
/// The mutable pointer to the underlying `publish_subscribe::Header`
pub type iox2_publish_subscribe_header_ptr_mut = *mut Header;

impl AssertNonNullHandle for iox2_publish_subscribe_header_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_publish_subscribe_header_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_publish_subscribe_header_h {
    type Target = *mut iox2_publish_subscribe_header_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_publish_subscribe_header_h_ref {
    type Target = *mut iox2_publish_subscribe_header_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END types definition

// BEGIN C API

/// This function needs to be called to destroy the publish_subscribe_header!
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_publish_subscribe_header_t`] can be re-used
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_drop(
    handle: iox2_publish_subscribe_header_h,
) {
    debug_assert!(!handle.is_null());

    let header = &mut *handle.as_type();
    core::ptr::drop_in_place(header.value.as_option_mut());

    (header.deleter)(header);
}

/// Returns the unique publisher id of the source of the sample.
///
/// # Arguments
///
/// * `handle` is valid, non-null and was initialized with
///   [`iox2_sample_header()`](crate::iox2_sample_header)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_publisher_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_publisher_id_h`].
///
/// # Safety
///
/// * `header_handle` is valid and non-null
/// * `id_struct_ptr` is either null or valid and non-null
/// * `id_handle_ptr` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_publisher_id(
    header_handle: iox2_publish_subscribe_header_h_ref,
    id_struct_ptr: *mut iox2_unique_publisher_id_t,
    id_handle_ptr: *mut iox2_unique_publisher_id_h,
) {
    header_handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_publisher_id_t) {}
    let mut deleter: fn(*mut iox2_unique_publisher_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_publisher_id_t::dealloc;
        storage_ptr = iox2_unique_publisher_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let header = &mut *header_handle.as_type();

    let id = header.value.as_ref().publisher_id();

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Returns the number of elements of the payload.
/// The element size is defined via this call when creating a new service
/// [`crate::iox2_service_builder_pub_sub_set_payload_type_details()`].
/// So if the payload is defined with alignment 8 and size 16 and this function returns 5. It
/// means that the payload consists of 5 elements of size 16 and every element is 8 byte aligned.
/// Therefore, the payload pointer points to a memory region with 5 * 16 = 80 bytes.
///
/// # Arguments
///
/// * `handle` is valid, non-null and was initialized with
///   [`iox2_sample_header()`](crate::iox2_sample_header)
///
/// # Safety
///
/// * `header_handle` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_number_of_elements(
    header_handle: iox2_publish_subscribe_header_h_ref,
) -> u64 {
    header_handle.assert_non_null();

    let header = &mut *header_handle.as_type();

    header.value.as_ref().number_of_elements()
}
// END C API
