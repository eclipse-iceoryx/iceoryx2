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
pub(crate) struct RawSample<Header, Metadata, Payload: ?Sized> {
    header: *const Header,
    metadata: *const Metadata,
    payload: *const Payload,
}

impl<Header, Metadata, Payload> RawSample<Header, Metadata, [Payload]> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `header` must be non-null.
    /// * `metadata` must be non-null.
    /// * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_slice_unchecked(
        header: *const Header,
        metadata: *const Metadata,
        payload: *const [Payload],
    ) -> Self {
        debug_assert!(
            !header.is_null() && !metadata.is_null() && !payload.is_null(),
            "RawSample::new_unchecked requires that the header-, metadata- and payload-pointer is non-null"
        );

        Self {
            header,
            metadata,
            payload,
        }
    }
}

impl<Header, Metadata, Payload: ?Sized> RawSample<Header, Metadata, Payload> {
    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        unsafe { &*self.header }
    }

    /// Acquires the underlying ,metadata as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_metadata_ref(&self) -> &Metadata {
        unsafe { &*self.metadata }
    }

    /// Acquires the underlying data as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_payload_ref(&self) -> &Payload {
        unsafe { &*self.payload }
    }
}

impl<Header, Metadata, Payload> RawSample<Header, Metadata, Payload> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `header` must be non-null.
    /// * `metadata` must be non-null.
    /// * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_unchecked(
        header: *const Header,
        metadata: *const Metadata,
        payload: *const Payload,
    ) -> Self {
        debug_assert!(
            !header.is_null() && !metadata.is_null() && !payload.is_null(),
            "RawSample::new_unchecked requires that the header-, metadata- and payload-pointer is non-null"
        );

        Self {
            header,
            metadata,
            payload,
        }
    }
}

impl<Header, Metadata, Payload> Clone for RawSample<Header, Metadata, Payload> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, Metadata, Payload> Copy for RawSample<Header, Metadata, Payload> {}

impl<Header, Metadata, Payload> fmt::Debug for RawSample<Header, Metadata, Payload> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RawSample<{}, {}, {}> {{ header: {:?}, metadata: {:?}, payload: {:?} }}",
            core::any::type_name::<Header>(),
            core::any::type_name::<Metadata>(),
            core::any::type_name::<Payload>(),
            self.header,
            self.metadata,
            self.payload
        )
    }
}

/// Contains the mutable pointer to the underlying header and payload of a sample.
#[repr(C)]
pub(crate) struct RawSampleMut<Header, Metadata, Payload: ?Sized> {
    header: *mut Header,
    metadata: *mut Metadata,
    payload: *mut Payload,
}

impl<Header, Metadata, Payload: ?Sized> RawSampleMut<Header, Metadata, Payload> {
    /// Creates a new `RawSampleMut`.
    ///
    /// # Safety
    ///
    ///  * `header` mut be non-null.
    ///  * `metadata` mut be non-null.
    ///  * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_unchecked(
        header: *mut Header,
        metadata: *mut Metadata,
        payload: *mut Payload,
    ) -> Self {
        debug_assert!(
            !header.is_null() && !metadata.is_null() && !payload.is_null(),
            "RawSampleMut::new_unchecked requires that the header-, metadata- and payload-pointer is non-null"
        );
        Self {
            header,
            metadata,
            payload,
        }
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
    pub(crate) fn as_metadata_ref(&self) -> &Metadata {
        unsafe { &*self.metadata }
    }

    /// Acquires the underlying payload as mutable reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_metadata_mut(&mut self) -> &mut Metadata {
        unsafe { &mut *self.metadata }
    }

    /// Acquires the underlying payload as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_payload_ref(&self) -> &Payload {
        unsafe { &*self.payload }
    }

    /// Acquires the underlying payload as mutable reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_payload_mut(&mut self) -> &mut Payload {
        unsafe { &mut *self.payload }
    }
}

impl<Header, Metadata, Payload> Clone for RawSampleMut<Header, Metadata, Payload> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, Metadata, Payload> Copy for RawSampleMut<Header, Metadata, Payload> {}

impl<Header, Metadata, Payload> fmt::Debug for RawSampleMut<Header, Metadata, Payload> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RawSampleMut<{}, {}, {}> {{ header: {:?}, metadata: {:?}, payload: {:?} }}",
            core::any::type_name::<Header>(),
            core::any::type_name::<Metadata>(),
            core::any::type_name::<Payload>(),
            self.header,
            self.metadata,
            self.payload
        )
    }
}
