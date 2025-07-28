// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! Trait which describes a form of pointer. Required to distinguish normal pointers from
//! relocatable pointers.

/// Trait which describes a form of pointer. Required to distinguish normal pointers from
/// relocatable pointers.
pub trait PointerTrait<T> {
    /// Return a pointer to the underlying const type
    ///
    /// # Safety
    ///
    ///  * Do not call this method when the pointer contains a null pointer.
    ///
    unsafe fn as_ptr(&self) -> *const T;

    /// Return a pointer to the underlying mutable type
    ///
    /// # Safety
    ///
    ///  * Do not call this method when the pointer contains a null pointer.
    ///
    unsafe fn as_mut_ptr(&mut self) -> *mut T;

    /// Indicates whether the pointer has been initialized.
    ///
    /// *Note:* This method should not be used when the pointer can be initialized to
    /// point to itself; it is allowed to report false negatives in this case.
    fn is_initialized(&self) -> bool {
        true
    }
}
