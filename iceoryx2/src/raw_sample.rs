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
use std::{alloc::Layout, mem::MaybeUninit};

use iceoryx2_bb_elementary::math::align;

fn aligned_header_size<Header, MessageType>() -> usize {
    align(
        core::mem::size_of::<Header>(),
        core::mem::align_of::<MessageType>(),
    )
}

fn get_layout<Header, MessageType>(number_of_elements: usize) -> Layout {
    unsafe {
        Layout::from_size_align_unchecked(
            align(
                aligned_header_size::<Header, MessageType>()
                    + core::mem::size_of::<MessageType>() * number_of_elements,
                core::mem::align_of::<Header>(),
            ),
            core::mem::align_of::<Header>(),
        )
    }
}

pub(crate) fn header_message_ptr<Header, MessageType>(
    raw_ptr: *const u8,
) -> (*const Header, *const u8) {
    let header_ptr = raw_ptr as *mut Header;
    let message_ptr = unsafe { raw_ptr.add(aligned_header_size::<Header, MessageType>()) };

    (header_ptr, message_ptr)
}

/// A `*const Message<Header, MessageType>` non-zero sample pointer to the message.
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
    /// `message` must be non-null.
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
    /// `raw_ptr` must be non-null.
    #[inline]
    pub(crate) unsafe fn new_unchecked(raw_ptr: *const u8) -> Self {
        debug_assert!(
            !raw_ptr.is_null(),
            "RawSample::new_unchecked requires that the raw-pointer is non-null"
        );

        let (header, message) = header_message_ptr::<Header, MessageType>(raw_ptr);
        Self {
            header,
            message: message as *const MessageType,
        }
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

/// A `*mut Message<Header, MessageType>` non-zero sample pointer to the message.
#[repr(C)]
pub(crate) struct RawSampleMut<Header, MessageType: ?Sized> {
    header: *mut Header,
    message: *mut MessageType,
}

impl<Header, MessageType> RawSampleMut<Header, MessageType> {
    pub(crate) fn layout() -> Layout {
        get_layout::<Header, MessageType>(1)
    }

    pub(crate) fn header_message_ptr(
        raw_ptr: *mut u8,
    ) -> (*mut Header, *mut MaybeUninit<MessageType>) {
        let (h, m) = header_message_ptr::<Header, MessageType>(raw_ptr);

        (h as *mut Header, m as *mut MaybeUninit<MessageType>)
    }
}

impl<Header, MessageType> RawSampleMut<Header, [MessageType]> {
    pub(crate) fn layout_slice(number_of_elements: usize) -> Layout {
        get_layout::<Header, MessageType>(number_of_elements)
    }

    pub(crate) fn header_slice_message_ptr(
        raw_ptr: *mut u8,
        number_of_elements: usize,
    ) -> (*mut Header, *mut [MaybeUninit<MessageType>]) {
        let (h, m) = header_message_ptr::<Header, MessageType>(raw_ptr);

        (h as *mut Header, unsafe {
            core::slice::from_raw_parts_mut(m as *mut MaybeUninit<MessageType>, number_of_elements)
        })
    }
}

impl<Header, MessageType: ?Sized> RawSampleMut<Header, MessageType> {
    /// Creates a new `RawSampleMut`.
    ///
    /// # Safety
    ///
    /// `message` must be non-null.
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

    /// Acquires the underlying header as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_header_mut(&self) -> &mut Header {
        unsafe { &mut *self.header }
    }

    /// Acquires the underlying data as reference.
    #[must_use]
    #[inline(always)]
    pub(crate) fn as_message_ref(&self) -> &MessageType {
        unsafe { &*self.message }
    }

    /// Acquires the underlying data as mut reference.
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
