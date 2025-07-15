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

use alloc::ffi::CString;
use core::ffi::c_char;

use iceoryx2::service::attribute::{Attribute, AttributeKey, AttributeSet};
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{
    iox2_attribute_h_ref, iox2_callback_context, iox2_callback_progression_e, AssertNonNullHandle,
    HandleToType,
};

// BEGIN types definition
#[repr(C)]
#[repr(align(8))] // alignment of Option<AttributeSet>
pub struct iox2_attribute_set_storage_t {
    internal: [u8; 5672], // magic number obtained with size_of::<Option<AttributeSet>>()
}

#[repr(C)]
#[iceoryx2_ffi(AttributeSet)]
pub struct iox2_attribute_set_t {
    pub value: iox2_attribute_set_storage_t,
    deleter: fn(*mut iox2_attribute_set_t),
}

pub struct iox2_attribute_set_h_t;
/// The owning handle for `iox2_attribute_set_t`. Passing the handle to an function transfers the ownership.
pub type iox2_attribute_set_h = *mut iox2_attribute_set_h_t;

/// The non-owning handle for `iox2_attribute_set_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_attribute_set_h_ref = *const iox2_attribute_set_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `AttributeSet`
pub type iox2_attribute_set_ptr = *const AttributeSet;
/// The mutable pointer to the underlying `AttributeSet`
pub type iox2_attribute_set_ptr_mut = *mut AttributeSet;

impl AssertNonNullHandle for iox2_attribute_set_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_attribute_set_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_attribute_set_h {
    type Target = *mut iox2_attribute_set_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_attribute_set_h_ref {
    type Target = *mut iox2_attribute_set_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

pub type iox2_attribute_set_get_callback =
    extern "C" fn(*const c_char, iox2_callback_context) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// This function create a new attribute_set by cloning an already existing one!
///
/// # Safety
///
/// * `struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_attribute_set_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `source_ptr` - Must be valid pointer to a [`iox2_attribute_set_ptr`].
/// * `handle_ptr` - An uninitialized or dangling [`iox2_attribute_set_h`] handle which will be initialized by this function call.
///
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_new_clone(
    struct_ptr: *mut iox2_attribute_set_t,
    source_ptr: iox2_attribute_set_ptr,
    handle_ptr: *mut iox2_attribute_set_h,
) {
    debug_assert!(!handle_ptr.is_null());
    debug_assert!(!source_ptr.is_null());

    *handle_ptr = core::ptr::null_mut();

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_attribute_set_t) {}
    let mut deleter: fn(*mut iox2_attribute_set_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_attribute_set_t::alloc();
        deleter = iox2_attribute_set_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    unsafe {
        (*struct_ptr).deleter = deleter;
    }

    unsafe {
        (*struct_ptr).value.init((*source_ptr).clone());
    }

    *handle_ptr = (*struct_ptr).as_handle();
}

/// This function needs to be called to destroy the attribute set!
///
/// # Safety
///
/// * `handle` - A valid [`iox2_attribute_set_h`] created with [`iox2_attribute_set_new_clone()`].
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_attribute_set_t`] can be re-used with a call to [`iox2_attribute_set_new_clone()`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_drop(handle: iox2_attribute_set_h) {
    debug_assert!(!handle.is_null());

    let attribute_set = &mut *handle.as_type();

    core::ptr::drop_in_place(attribute_set.value.as_option_mut());
    (attribute_set.deleter)(attribute_set);
}

/// This function casts a [`iox2_attribute_set_h`] into a [`iox2_attribute_set_ptr`]
///
/// Returns a [`iox2_attribute_set_ptr`]
///
/// # Safety
///
/// * `handle` obtained by [`iox2_attribute_set_new_clone()`]
/// * The `handle` must be a valid handle.
/// * The `handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_attribute_set_ptr(
    handle: iox2_attribute_set_h,
) -> iox2_attribute_set_ptr {
    debug_assert!(!handle.is_null());

    (*handle.as_type()).value.as_ref()
}

/// Returns the number of attributes in the attribute set.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_number_of_attributes(
    handle: iox2_attribute_set_ptr,
) -> usize {
    debug_assert!(!handle.is_null());

    (*handle).iter().len()
}

/// Returns a [`iox2_attribute_h_ref`] to the attribute stored at the provided index.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `index` < [`iox2_attribute_set_number_of_attributes()`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_index(
    handle: iox2_attribute_set_ptr,
    index: usize,
) -> iox2_attribute_h_ref {
    debug_assert!(!handle.is_null());
    debug_assert!(index < iox2_attribute_set_number_of_attributes(handle));

    (&(&(*handle))[index] as *const Attribute).cast()
}

/// Returns the number of values stored under a specific key. If the key does not exist it
/// returns 0.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * `key` must be non-zero and contain a null-terminated string
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_number_of_key_values(
    handle: iox2_attribute_set_ptr,
    key: *const c_char,
) -> usize {
    debug_assert!(!handle.is_null());
    debug_assert!(!key.is_null());

    let key = AttributeKey::from_c_str(key);
    if key.is_err() {
        return 0;
    }
    let key = key.unwrap();

    (*handle).number_of_key_values(&key)
}

/// Returns a value of a key at a specific index. The index enumerates the values of the key
/// if the key has multiple values. The values are always stored at the same position during
/// the lifetime of the service but they can change when the process is recreated by another
/// process when the system restarts.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * `key` must be non-zero and contain a null-terminated string
/// * `buffer` must point to a valid memory location
/// * `buffer_len` must define the length of the memory pointed by `buffer`
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_key_value(
    handle: iox2_attribute_set_ptr,
    key: *const c_char,
    index: usize,
    buffer: *mut c_char,
    buffer_len: usize,
    has_value: *mut bool,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!key.is_null());
    debug_assert!(!buffer.is_null());
    debug_assert!(0 < buffer_len);

    *has_value = false;
    let key = AttributeKey::from_c_str(key);
    if key.is_err() {
        return;
    }
    let key = key.unwrap();

    if let Some(v) = (*handle).key_value(&key, index) {
        if let Ok(value) = CString::new(v.as_bytes()) {
            core::ptr::copy_nonoverlapping(
                value.as_ptr(),
                buffer,
                buffer_len.min(value.count_bytes() + 1 /* null terminator */),
            );
            *has_value = true;
        }
    }
}

/// Calls the provided callback for every value that is owned by the provided key.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `key` must be a valid null-terminated string.
/// * The `callback` must point to a function with the required signature.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_iter_key_values(
    handle: iox2_attribute_set_ptr,
    key: *const c_char,
    callback: iox2_attribute_set_get_callback,
    callback_ctx: iox2_callback_context,
) {
    debug_assert!(!handle.is_null());

    let key = AttributeKey::from_c_str(key);
    if key.is_err() {
        return;
    }
    let key = key.unwrap();

    (*handle).iter_key_values(&key, |value| {
        if let Ok(value) = CString::new(value.as_bytes()) {
            callback(value.as_ptr(), callback_ctx).into()
        } else {
            CallbackProgression::Continue
        }
    });
}
// END C API
