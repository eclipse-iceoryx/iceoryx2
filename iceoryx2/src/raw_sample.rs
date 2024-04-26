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
pub(crate) struct RawSample<Header, PayloadType: ?Sized> {
    header: *const Header,
    payload: *const PayloadType,
}

impl<Header, PayloadType> RawSample<Header, [PayloadType]> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `header` must be non-null.
    /// * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_slice_unchecked(
        header: *const Header,
        payload: *const [PayloadType],
    ) -> Self {
        debug_assert!(
            !header.is_null() && !payload.is_null(),
            "RawSample::new_unchecked requires that the header- and payload-pointer is non-null"
        );

        Self { header, payload }
    }
}

impl<Header, PayloadType: ?Sized> RawSample<Header, PayloadType> {
    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        unsafe { &*self.header }
    }

    /// Acquires the underlying data as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_payload_ref(&self) -> &PayloadType {
        unsafe { &*self.payload }
    }
}

impl<Header, PayloadType> RawSample<Header, PayloadType> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `header` must be non-null.
    /// * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_unchecked(header: *const Header, payload: *const PayloadType) -> Self {
        debug_assert!(
            !header.is_null() && !payload.is_null(),
            "RawSample::new_unchecked requires that the header- and payload-pointer is non-null"
        );

        Self { header, payload }
    }
}

impl<Header, PayloadType> Clone for RawSample<Header, PayloadType> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, PayloadType> Copy for RawSample<Header, PayloadType> {}

impl<Header: fmt::Debug, PayloadType: fmt::Debug> fmt::Debug for RawSample<Header, PayloadType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.payload, f)
    }
}

impl<Header, PayloadType> fmt::Pointer for RawSample<Header, PayloadType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.payload, f)
    }
}

/// Contains the mutable pointer to the underlying header and payload of a sample.
#[repr(C)]
pub(crate) struct RawSampleMut<Header, PayloadType: ?Sized> {
    header: *mut Header,
    payload: *mut PayloadType,
}

impl<Header, PayloadType: ?Sized> RawSampleMut<Header, PayloadType> {
    /// Creates a new `RawSampleMut`.
    ///
    /// # Safety
    ///
    ///  * `header` mut be non-null.
    ///  * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_unchecked(header: *mut Header, payload: *mut PayloadType) -> Self {
        debug_assert!(
            !header.is_null() && !payload.is_null(),
            "RawSampleMut::new_unchecked requires that the payload pointer is non-null"
        );
        Self { header, payload }
    }

    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        unsafe { &*self.header }
    }

    /// Acquires the underlying payload as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_payload_ref(&self) -> &PayloadType {
        unsafe { &*self.payload }
    }

    /// Acquires the underlying payload as mutable reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_payload_mut(&mut self) -> &mut PayloadType {
        unsafe { &mut *self.payload }
    }
}

impl<Header, PayloadType> Clone for RawSampleMut<Header, PayloadType> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, PayloadType> Copy for RawSampleMut<Header, PayloadType> {}

impl<Header: fmt::Debug, PayloadType: fmt::Debug> fmt::Debug for RawSampleMut<Header, PayloadType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_header_ref(), f)
    }
}

impl<Header, PayloadType> fmt::Pointer for RawSampleMut<Header, PayloadType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_header_ref(), f)
    }
}
