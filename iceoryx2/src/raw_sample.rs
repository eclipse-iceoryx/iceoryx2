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

use crate::message::Message;

use core::fmt;

/// A `*const Message<Header, Data>` non-zero sample pointer to the message.
#[repr(transparent)]
pub(crate) struct RawSample<Header, Data> {
    message: *const Message<Header, Data>,
}

impl<Header, Data> RawSample<Header, Data> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// `message` must be non-null.
    #[inline]
    pub(crate) unsafe fn new_unchecked(message: *const Message<Header, Data>) -> Self {
        debug_assert!(
            !message.is_null(),
            "RawSample::new_unchecked requires that the message pointer is non-null"
        );
        Self { message }
    }

    /// Creates a new `RawSample`.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn new(message: *const Message<Header, Data>) -> Option<Self> {
        if !message.is_null() {
            // SAFETY: `message` pointer is checked to be non-null
            Some(unsafe { Self::new_unchecked(message) })
        } else {
            None
        }
    }

    /// Acquires the underlying message as `*const` pointer.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_ptr(self) -> *const Message<Header, Data> {
        self.message
    }

    /// Acquires the underlying message as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_ref(&self) -> &Message<Header, Data> {
        // SAFETY: `self.as_ptr()` returns a non-null ptr and `Data` is either the actual message type or wrapped by a `MaybeUninit` which makes a reference to `Message::data` safe
        unsafe { &(*self.as_ptr()) }
    }

    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        &self.as_ref().header
    }

    /// Acquires the underlying data as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_data_ref(&self) -> &Data {
        &self.as_ref().data
    }
}

impl<Header, Data> Clone for RawSample<Header, Data> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, Data> Copy for RawSample<Header, Data> {}

impl<Header: fmt::Debug, Data: fmt::Debug> fmt::Debug for RawSample<Header, Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.message, f)
    }
}

impl<Header, Data> fmt::Pointer for RawSample<Header, Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.message, f)
    }
}

/// A `*mut Message<Header, Data>` non-zero sample pointer to the message.
#[repr(transparent)]
pub(crate) struct RawSampleMut<Header, Data> {
    message: *mut Message<Header, Data>,
}

impl<Header, Data> RawSampleMut<Header, Data> {
    /// Creates a new `RawSampleMut`.
    ///
    /// # Safety
    ///
    /// `message` must be non-null.
    #[inline]
    pub(crate) unsafe fn new_unchecked(message: *mut Message<Header, Data>) -> Self {
        debug_assert!(
            !message.is_null(),
            "RawSampleMut::new_unchecked requires that the message pointer is non-null"
        );
        Self { message }
    }

    /// Creates a new `RawSampleMut`.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn new(message: *mut Message<Header, Data>) -> Option<Self> {
        if !message.is_null() {
            // SAFETY: `message` pointer is checked to be non-null
            Some(unsafe { Self::new_unchecked(message) })
        } else {
            None
        }
    }

    /// Acquires the underlying message as `*const` pointer.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_ptr(self) -> *const Message<Header, Data> {
        self.message
    }

    /// Acquires the underlying message as `*mut` pointer.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_mut_ptr(self) -> *mut Message<Header, Data> {
        self.message
    }

    /// Acquires the underlying message as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_ref(&self) -> &Message<Header, Data> {
        // SAFETY: `self.as_ptr()` returns a non-null ptr and `Data` is either the actual message type or wrapped by a `MaybeUninit` which makes a reference to `Message::data` safe
        unsafe { &(*self.as_ptr()) }
    }

    /// Acquires the underlying message as mut reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_mut(&mut self) -> &mut Message<Header, Data> {
        // SAFETY: `self.as_ptr()` returns a non-null ptr and `Data` is either the actual message type or wrapped by a `MaybeUninit` which makes a reference to `Message::data` safe
        unsafe { &mut (*self.as_mut_ptr()) }
    }

    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        &self.as_ref().header
    }

    /// Acquires the underlying data as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_data_ref(&self) -> &Data {
        &self.as_ref().data
    }

    /// Acquires the underlying data as mut reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_data_mut(&mut self) -> &mut Data {
        &mut self.as_mut().data
    }
}

impl<Header, Data> Clone for RawSampleMut<Header, Data> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, Data> Copy for RawSampleMut<Header, Data> {}

impl<Header: fmt::Debug, Data: fmt::Debug> fmt::Debug for RawSampleMut<Header, Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<Header, Data> fmt::Pointer for RawSampleMut<Header, Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}
