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

use core::ffi::CStr;

/// Trait for types that can be represented as a C-style string.
///
/// This trait provides a method to obtain a reference to a static C-style string
/// representation of the implementing type.
///
/// # Safety
///
/// Implementations of this trait must ensure that the returned `CStr` is valid
/// and remains valid for the entire lifetime of the program.
pub trait AsCStr {
    fn as_const_cstr(&self) -> &'static CStr;
}
