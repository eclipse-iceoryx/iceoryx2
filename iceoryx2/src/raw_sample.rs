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
pub(crate) struct RawSample<Header, UserHeader, Payload: ?Sized> {
    header: *const Header,
    user_header: *const UserHeader,
    payload: *const Payload,
}

impl<Header, UserHeader, Payload> RawSample<Header, UserHeader, [Payload]> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `header` must be non-null.
    /// * `user_header` must be non-null.
    /// * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_slice_unchecked(
        header: *const Header,
        user_header: *const UserHeader,
        payload: *const [Payload],
    ) -> Self {
        debug_assert!(
            !header.is_null() && !user_header.is_null() && !payload.is_null(),
            "RawSample::new_unchecked requires that the header-, user_header- and payload-pointer is non-null"
        );

        Self {
            header,
            user_header,
            payload,
        }
    }
}

impl<Header, UserHeader, Payload: ?Sized> RawSample<Header, UserHeader, Payload> {
    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_ref(&self) -> &Header {
        unsafe { &*self.header }
    }

    /// Acquires the underlying ,user_header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_user_header_ref(&self) -> &UserHeader {
        unsafe { &*self.user_header }
    }

    /// Acquires the underlying data as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_payload_ref(&self) -> &Payload {
        unsafe { &*self.payload }
    }
}

impl<Header, UserHeader, Payload> RawSample<Header, UserHeader, Payload> {
    /// Creates a new `RawSample`.
    ///
    /// # Safety
    ///
    /// * `header` must be non-null.
    /// * `user_header` must be non-null.
    /// * `payload` must be non-null.
    ///
    #[inline]
    pub(crate) unsafe fn new_unchecked(
        header: *const Header,
        user_header: *const UserHeader,
        payload: *const Payload,
    ) -> Self {
        debug_assert!(
            !header.is_null() && !user_header.is_null() && !payload.is_null(),
            "RawSample::new_unchecked requires that the header-, user_header- and payload-pointer is non-null"
        );

        Self {
            header,
            user_header,
            payload,
        }
    }
}

impl<Header, UserHeader, Payload> Clone for RawSample<Header, UserHeader, Payload> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Header, UserHeader, Payload> Copy for RawSample<Header, UserHeader, Payload> {}

impl<Header, UserHeader, Payload: ?Sized> fmt::Debug for RawSample<Header, UserHeader, Payload> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RawSample<{}, {}, {}> {{ header: {:?}, user_header: {:?}, payload: {:?} }}",
            core::any::type_name::<Header>(),
            core::any::type_name::<UserHeader>(),
            core::any::type_name::<Payload>(),
            self.header,
            self.user_header,
            self.payload
        )
    }
}
