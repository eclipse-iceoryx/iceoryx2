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

use crate::api::{AssertNonNullHandle, HandleToType, IOX2_OK};

use iceoryx2::prelude::*;
use iceoryx2::service::attribute::{AttributeKey, AttributeValue};
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use alloc::ffi::CString;
use core::ffi::c_int;
use core::{ffi::c_char, mem::ManuallyDrop};

use super::iox2_attribute_set_ptr;

// BEGIN type definition

#[repr(C)]
pub(crate) struct AttributeVerifierType(pub(crate) ManuallyDrop<AttributeVerifier>);

impl AttributeVerifierType {
    fn new() -> Self {
        Self(ManuallyDrop::new(AttributeVerifier::new()))
    }

    fn from(value: AttributeVerifier) -> Self {
        Self(ManuallyDrop::new(value))
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<AttributeVerifier>
pub struct iox2_attribute_verifier_storage_t {
    internal: [u8; 6984], // magic number obtained with size_of::<Option<AttributeVerifier>>()
}

#[repr(C)]
#[iceoryx2_ffi(AttributeVerifierType)]
pub struct iox2_attribute_verifier_t {
    pub value: iox2_attribute_verifier_storage_t,
    deleter: fn(*mut iox2_attribute_verifier_t),
}

pub struct iox2_attribute_verifier_h_t;
/// The owning handle for `iox2_attribute_verifier_t`. Passing the handle to an function transfers the ownership.
pub type iox2_attribute_verifier_h = *mut iox2_attribute_verifier_h_t;

/// The non-owning handle for `iox2_attribute_verifier_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_attribute_verifier_h_ref = *const iox2_attribute_verifier_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeName`
pub type iox2_attribute_verifier_ptr = *const AttributeVerifier;
/// The mutable pointer to the underlying `AttributeVerifier`
pub type iox2_attribute_verifier_ptr_mut = *mut AttributeVerifier;

impl AssertNonNullHandle for iox2_attribute_verifier_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_attribute_verifier_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_attribute_verifier_h {
    type Target = *mut iox2_attribute_verifier_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_attribute_verifier_h_ref {
    type Target = *mut iox2_attribute_verifier_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Creates a new [`iox2_attribute_verifier_h`]. It must be cleaned up with
/// [`iox2_attribute_verifier_drop()`].
/// If the `struct_ptr` is null, then the function will allocate memory.
///
/// # Safety
///
/// * The `handle_ptr` must point to an uninitialized [`iox2_attribute_verifier_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_new(
    struct_ptr: *mut iox2_attribute_verifier_t,
    handle_ptr: *mut iox2_attribute_verifier_h,
) -> c_int {
    debug_assert!(!handle_ptr.is_null());

    *handle_ptr = core::ptr::null_mut();

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_attribute_verifier_t) {}
    let mut deleter: fn(*mut iox2_attribute_verifier_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_attribute_verifier_t::alloc();
        deleter = iox2_attribute_verifier_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    unsafe { (*struct_ptr).deleter = deleter };
    unsafe { (*struct_ptr).value.init(AttributeVerifierType::new()) };

    *handle_ptr = (*struct_ptr).as_handle();

    IOX2_OK
}

/// Deletes a [`iox2_attribute_verifier_h`]. It must be created with
/// [`iox2_attribute_verifier_new()`].
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_drop(handle: iox2_attribute_verifier_h) {
    debug_assert!(!handle.is_null());

    let attribute_verifier = &mut *handle.as_type();

    ManuallyDrop::drop(&mut attribute_verifier.value.as_mut().0);
    (attribute_verifier.deleter)(attribute_verifier);
}

/// Defines a attribute (key / value pair) that is required.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
/// * The `key` must point to a valid null-terminated string.
/// * The `value` must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_require(
    handle: iox2_attribute_verifier_h_ref,
    key: *const c_char,
    value: *const c_char,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!key.is_null());
    debug_assert!(!value.is_null());

    let key = AttributeKey::from_c_str(key);
    let value = AttributeValue::from_c_str(value);

    debug_assert!(key.is_ok() && value.is_ok());

    let attribute_verifier_struct = &mut *handle.as_type();
    let attribute_verifier = ManuallyDrop::take(&mut attribute_verifier_struct.value.as_mut().0);
    attribute_verifier_struct.set(AttributeVerifierType::from(
        attribute_verifier.require(&key.unwrap(), &value.unwrap()),
    ));
}

/// Defines a key that must be present.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
/// * The `key` must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_require_key(
    handle: iox2_attribute_verifier_h_ref,
    key: *const c_char,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!key.is_null());

    let key = AttributeKey::from_c_str(key);

    debug_assert!(key.is_ok());

    let attribute_verifier_struct = &mut *handle.as_type();
    let attribute_verifier = ManuallyDrop::take(&mut attribute_verifier_struct.value.as_mut().0);
    attribute_verifier_struct.set(AttributeVerifierType::from(
        attribute_verifier.require_key(&key.unwrap()),
    ));
}

/// Returnes a [`iox2_attribute_set_ptr`] to the underlying attribute set.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
/// * The `handle` must live at least as long as the returned [`iox2_attribute_set_ptr`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_attributes(
    handle: iox2_attribute_verifier_h_ref,
) -> iox2_attribute_set_ptr {
    debug_assert!(!handle.is_null());

    let attribute_verifier_struct = &mut *handle.as_type();
    attribute_verifier_struct
        .value
        .as_ref()
        .0
        .required_attributes()
}

/// Verifies if the [`iox2_attribute_set_ptr`] contains all required keys and key-value pairs.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
/// * The `rhs` must be valid.
/// * `incompatible_key_buffer` must be either null or point to a valid memory location of size
///   `incompatible_key_buffer_len`
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_verify_requirements(
    handle: iox2_attribute_verifier_h_ref,
    rhs: iox2_attribute_set_ptr,
    incompatible_key_buffer: *mut c_char,
    incompatible_key_buffer_len: usize,
) -> bool {
    debug_assert!(!handle.is_null());
    debug_assert!(!rhs.is_null());

    let attribute_verifier_struct = &mut *handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    match attribute_verifier.verify_requirements(&*rhs) {
        Ok(()) => true,
        Err(incompatible_key) => {
            if let Ok(incompatible_key) = CString::new(incompatible_key) {
                if incompatible_key_buffer_len != 0 && !incompatible_key_buffer.is_null() {
                    core::ptr::copy_nonoverlapping(
                        incompatible_key.as_bytes_with_nul().as_ptr(),
                        incompatible_key_buffer.cast(),
                        incompatible_key_buffer_len.min(incompatible_key.as_bytes_with_nul().len()),
                    );
                }
            }
            false
        }
    }
}

/// Returns the number of required keys.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_number_of_keys(
    handle: iox2_attribute_verifier_h_ref,
) -> usize {
    debug_assert!(!handle.is_null());
    let attribute_verifier_struct = &mut *handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;
    attribute_verifier.required_keys().len()
}

/// Returns the length of a required key at a specific key index.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
/// * `key_index` < [`iox2_attribute_verifier_number_of_keys()`]
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_key_len(
    handle: iox2_attribute_verifier_h_ref,
    key_index: usize,
) -> usize {
    debug_assert!(!handle.is_null());
    let attribute_verifier_struct = &mut *handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    debug_assert!(key_index < attribute_verifier.required_keys().len());
    attribute_verifier.required_keys()[key_index].len()
}

/// Copies the key value at a specific key index into the provided buffer.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_verifier_h`].
/// * `key_index` < [`iox2_attribute_verifier_number_of_keys()`]
/// * `key_value_buffer` must point to a valid memory location of size `key_value_buffer_len`.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_verifier_key(
    handle: iox2_attribute_verifier_h_ref,
    key_index: usize,
    key_value_buffer: *mut c_char,
    key_value_buffer_len: usize,
) -> usize {
    debug_assert!(!handle.is_null());
    debug_assert!(!key_value_buffer.is_null());
    let attribute_verifier_struct = &mut *handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    debug_assert!(key_index < attribute_verifier.required_keys().len());

    if let Ok(key) = CString::new(attribute_verifier.required_keys()[key_index].as_bytes()) {
        let copied_length = key_value_buffer_len.min(key.as_bytes_with_nul().len());

        core::ptr::copy_nonoverlapping(
            key.as_bytes_with_nul().as_ptr(),
            key_value_buffer.cast(),
            copied_length,
        );

        copied_length
    } else {
        0
    }
}
// END C API
