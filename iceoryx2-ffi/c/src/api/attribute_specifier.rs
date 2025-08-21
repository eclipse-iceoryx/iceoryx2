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

use core::ffi::c_int;
use core::{ffi::c_char, mem::ManuallyDrop};

use super::iox2_attribute_set_ptr;

// BEGIN type definition

#[repr(C)]
pub(crate) struct AttributeSpecifierType(pub(crate) ManuallyDrop<AttributeSpecifier>);

impl AttributeSpecifierType {
    fn new() -> Self {
        Self(ManuallyDrop::new(AttributeSpecifier::new()))
    }

    fn from(value: AttributeSpecifier) -> Self {
        Self(ManuallyDrop::new(value))
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<AttributeSpecifier>
pub struct iox2_attribute_specifier_storage_t {
    internal: [u8; 5672], // magic number obtained with size_of::<Option<AttributeSpecifier>>()
}

#[repr(C)]
#[iceoryx2_ffi(AttributeSpecifierType)]
pub struct iox2_attribute_specifier_t {
    pub value: iox2_attribute_specifier_storage_t,
    deleter: fn(*mut iox2_attribute_specifier_t),
}

pub struct iox2_attribute_specifier_h_t;
/// The owning handle for `iox2_attribute_specifier_t`. Passing the handle to an function transfers the ownership.
pub type iox2_attribute_specifier_h = *mut iox2_attribute_specifier_h_t;

/// The non-owning handle for `iox2_attribute_specifier_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_attribute_specifier_h_ref = *const iox2_attribute_specifier_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeName`
pub type iox2_attribute_specifier_ptr = *const AttributeSpecifier;
/// The mutable pointer to the underlying `AttributeSpecifier`
pub type iox2_attribute_specifier_ptr_mut = *mut AttributeSpecifier;

impl AssertNonNullHandle for iox2_attribute_specifier_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_attribute_specifier_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_attribute_specifier_h {
    type Target = *mut iox2_attribute_specifier_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_attribute_specifier_h_ref {
    type Target = *mut iox2_attribute_specifier_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Creates a new [`iox2_attribute_specifier_h`]. It must be cleaned up with
/// [`iox2_attribute_specifier_drop()`].
/// If the `struct_ptr` is null, then the function will allocate memory.
///
/// # Safety
///
/// * The `handle_ptr` must point to an uninitialized [`iox2_attribute_specifier_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_specifier_new(
    struct_ptr: *mut iox2_attribute_specifier_t,
    handle_ptr: *mut iox2_attribute_specifier_h,
) -> c_int {
    debug_assert!(!handle_ptr.is_null());

    *handle_ptr = core::ptr::null_mut();

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_attribute_specifier_t) {}
    let mut deleter: fn(*mut iox2_attribute_specifier_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_attribute_specifier_t::alloc();
        deleter = iox2_attribute_specifier_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    unsafe { (*struct_ptr).deleter = deleter };
    unsafe { (*struct_ptr).value.init(AttributeSpecifierType::new()) };

    *handle_ptr = (*struct_ptr).as_handle();

    IOX2_OK
}

/// Deletes a [`iox2_attribute_specifier_h`]. It must be created with
/// [`iox2_attribute_specifier_new()`].
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_specifier_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_specifier_drop(handle: iox2_attribute_specifier_h) {
    debug_assert!(!handle.is_null());

    let attribute_specifier = &mut *handle.as_type();

    ManuallyDrop::drop(&mut attribute_specifier.value.as_mut().0);
    (attribute_specifier.deleter)(attribute_specifier);
}

/// Defines a attribute (key / value pair).
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_specifier_h`].
/// * The `key` must point to a valid null-terminated string.
/// * The `value` must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_specifier_define(
    handle: iox2_attribute_specifier_h_ref,
    key: *const c_char,
    value: *const c_char,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!key.is_null());
    debug_assert!(!value.is_null());

    let key = AttributeKey::from_c_str(key);
    let value = AttributeValue::from_c_str(value);

    debug_assert!(key.is_ok() && value.is_ok());

    let attribute_specifier_struct = &mut *handle.as_type();
    let attribute_specifier = ManuallyDrop::take(&mut attribute_specifier_struct.value.as_mut().0);
    attribute_specifier_struct.set(AttributeSpecifierType::from(
        attribute_specifier.define(&key.unwrap(), &value.unwrap()),
    ));
}

/// Returnes a [`iox2_attribute_set_ptr`] to the underlying attribute set.
///
/// # Safety
///
/// * The `handle` must point to an initialized [`iox2_attribute_specifier_h`].
/// * The `handle` must live at least as long as the returned [`iox2_attribute_set_ptr`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_specifier_attributes(
    handle: iox2_attribute_specifier_h_ref,
) -> iox2_attribute_set_ptr {
    debug_assert!(!handle.is_null());

    let attribute_specifier_struct = &mut *handle.as_type();
    attribute_specifier_struct.value.as_ref().0.attributes()
}
// END C API
