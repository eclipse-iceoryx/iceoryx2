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

use core::fmt;

/// Contains the pointer to the underlying header and payload of a sample.
#[repr(C)]
pub(crate) struct RawSample<Header, MessageType: ?Sized> {
    header: *const Header,
    message: *const MessageType,
}

impl<Header, MessageType> RawSample<Header, [MessageType]> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `header` must be non-null.
    /// * `message` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_slice_unchecked(
        header: *const Header,
        message: *const [MessageType],
    ) -> Self {
        debug_assert!(
            !header.is_null() && !message.is_null(),
            "RawSample::new_unchecked requires that the header- and message-pointer is non-null"
        );

        Self { header, message }
    }
}

impl<Header, MessageType: ?Sized> RawSample<Header, MessageType> {
    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        unsafe { &*self.header }
    }

    /// Acquires the underlying data as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_message_ref(&self) -> &MessageType {
        unsafe { &*self.message }
    }
}

impl<Header, MessageType> RawSample<Header, MessageType> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `raw_ptr` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_unchecked(header: *const Header, message: *const MessageType) -> Self {
        debug_assert!(
            !header.is_null() && !message.is_null(),
            "RawSample::new_unchecked requires that the header- and message-pointer is non-null"
        );

        Self { header, message }
    }
}

impl<Header, MessageType> Clone for RawSample<Header, MessageType> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, MessageType> Copy for RawSample<Header, MessageType> {}

impl<Header: fmt::Debug, MessageType: fmt::Debug> fmt::Debug for RawSample<Header, MessageType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.message, f)
    }
}

impl<Header, MessageType> fmt::Pointer for RawSample<Header, MessageType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.message, f)
    }
}

/// Contains the mutable pointer to the underlying header and payload of a sample.
#[repr(C)]
pub(crate) struct RawSampleMut<Header, MessageType: ?Sized> {
    header: *mut Header,
    message: *mut MessageType,
}

impl<Header, MessageType: ?Sized> RawSampleMut<Header, MessageType> {
    /// Creates a new `RawSampleMut`.
    ///
    /// # Safety
    ///
    ///  * `header` mut be non-null.
    ///  * `message` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_unchecked(header: *mut Header, message: *mut MessageType) -> Self {
        debug_assert!(
            !header.is_null() && !message.is_null(),
            "RawSampleMut::new_unchecked requires that the message pointer is non-null"
        );
        Self { header, message }
    }

    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        unsafe { &*self.header }
    }

    /// Acquires the underlying message as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_message_ref(&self) -> &MessageType {
        unsafe { &*self.message }
    }

    /// Acquires the underlying message as mutable reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_message_mut(&mut self) -> &mut MessageType {
        unsafe { &mut *self.message }
    }
}

impl<Header, MessageType> Clone for RawSampleMut<Header, MessageType> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, MessageType> Copy for RawSampleMut<Header, MessageType> {}

impl<Header: fmt::Debug, MessageType: fmt::Debug> fmt::Debug for RawSampleMut<Header, MessageType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_header_ref(), f)
    }
}

impl<Header, MessageType> fmt::Pointer for RawSampleMut<Header, MessageType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_header_ref(), f)
    }
}
