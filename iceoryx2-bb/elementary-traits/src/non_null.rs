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

use core::ptr::NonNull;

// TODO: #1613 - Temporary addition, remove when iceoryx2 MSRV >= 1.89

/// Trait for creating a `NonNull<T>` from a reference.
///
/// This trait provides a method to create a `NonNull<T>` from an existing
/// reference. This method is a replication from `core::ptr::NonNull::from_ref(&T)`
/// that will be introduced with Rust version 1.89
///
/// # Safety
///
/// Since a reference cannot be null we can safely use `core::ptr::NonNull::new_unchecked`
/// to create and return the NonNull
pub trait NonNullCompat<T> {
    fn iox2_from_ref(r: &T) -> NonNull<T>;
    fn iox2_from_mut(r: &mut T) -> NonNull<T>;
}

impl<T> NonNullCompat<T> for NonNull<T> {
    fn iox2_from_ref(r: &T) -> Self {
        // SAFETY: A reference cannot be null.
        unsafe { Self::new_unchecked(r as *const _ as *mut T) }
    }

    fn iox2_from_mut(r: &mut T) -> Self {
        // SAFETY: A reference cannot be null.
        unsafe { Self::new_unchecked(r as *mut T) }
    }
}
