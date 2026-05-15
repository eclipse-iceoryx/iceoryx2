// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use crate::non_null::NonNullCompat;
use core::ptr::NonNull;

/// **Only for testing purposes!**
///
/// Marks types that can abandon resources required for cleanup tests.
/// The system resource is abandon but the process local internal constructs
/// are still cleaned up properly. Those internal constructs could be:
///
/// * file descriptors
/// * memory mappings
/// * ...
pub trait Abandonable: Sized {
    fn abandon(mut self) {
        unsafe { Self::abandon_in_place(NonNull::iox2_from_mut(&mut self)) };
        core::mem::forget(self);
    }

    /// Abandon a resource in place. Shall be used when a struct of multiple resources
    /// shall be abandoned and the resources cannot be moved out of the struct.
    ///
    /// # Safety
    ///
    /// * `this` must pointing to a valid, constructed `Self`.
    /// * `this` must be non-null and properly aligned
    /// * `this` must not be dropped before.
    /// * `this` cannot be used after this operation, you should most likely call
    ///   [`core::mem::forget()`] afterwards.
    ///
    unsafe fn abandon_in_place(this: NonNull<Self>);
}
