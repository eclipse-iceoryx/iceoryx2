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

use std::rc::Rc;

use core::alloc::Layout;
use core::ffi::c_void;

use iceoryx2_link_adapter::{Region, SampleBytesRef, TranscodeError, Transcoder};
use iceoryx2_log::{fail, origin};
use r2r_rcl::{
    RMW_RET_OK, rcutils_allocator_t, rcutils_get_default_allocator, rmw_deserialize, rmw_serialize,
    rmw_serialized_message_t,
};

use crate::typesupport::TypeSupport;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TranscodeFailure {
    /// The payload is not the size of the C struct.
    PayloadSize,
    Serialize,
    Deserialize,
}

impl core::fmt::Display for TranscodeFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranscodeFailure::{self:?}")
    }
}

impl core::error::Error for TranscodeFailure {}

/// Converts between a ROS 2 type's C struct and its CDR wire form through
/// the rmw (de)serializer.
#[derive(Debug)]
pub struct CdrTranscoder {
    pub(super) type_name: String,
    pub(super) type_support: Rc<TypeSupport>,
    /// The layout of the type's C struct.
    pub(super) layout: Layout,
}

impl<'a> Transcoder<SampleBytesRef<'a>> for CdrTranscoder {
    type Error = TranscodeFailure;

    fn encode<R: Region>(
        &self,
        local: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("CdrTranscoder::encode");

        // The payload is the C struct the rmw serializes from.
        let payload = local.payload;
        if payload.len() != self.layout.size() {
            fail!(
                from origin,
                with TranscodeError::Transcoder(TranscodeFailure::PayloadSize),
                "Payload of {} bytes is not the {} bytes of type '{}'",
                payload.len(), self.layout.size(), self.type_name
            );
        }
        debug_assert!(
            (payload.as_ptr() as usize).is_multiple_of(self.layout.align()),
            "the payload must be aligned to the C struct"
        );

        // The rmw allocates the wire form straight into the region provided.
        let mut serialized = rmw_serialized_message_t {
            buffer: core::ptr::null_mut(),
            buffer_length: 0,
            buffer_capacity: 0,
            allocator: region_allocator(into),
        };
        let ret = unsafe {
            rmw_serialize(
                payload.as_ptr().cast::<c_void>(),
                self.type_support.handle(),
                &mut serialized,
            )
        };
        if ret != RMW_RET_OK as i32 {
            fail!(
                from origin,
                with TranscodeError::Transcoder(TranscodeFailure::Serialize),
                "The rmw failed to serialize a payload of type '{}'", self.type_name
            );
        }

        // Set the size to the serialized length.
        fail!(
            from origin,
            when into.for_length(serialized.buffer_length),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} serialized bytes of type '{}'",
            serialized.buffer_length, self.type_name
        );

        Ok(())
    }

    fn decode<R: Region>(
        &self,
        wire: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("CdrTranscoder::decode");

        // The region takes the size of the C struct.
        let wire = wire.payload;
        let destination = fail!(
            from origin,
            when into.for_length(self.layout.size()),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} bytes of type '{}'",
            self.layout.size(), self.type_name
        );
        debug_assert!(
            (destination.as_ptr() as usize).is_multiple_of(self.layout.align()),
            "the payload must be aligned to the C struct"
        );

        // The rmw reads the wire form in place, the allocator is never called.
        let serialized = rmw_serialized_message_t {
            buffer: wire.as_ptr().cast_mut(),
            buffer_length: wire.len(),
            buffer_capacity: wire.len(),
            allocator: unsafe { rcutils_get_default_allocator() },
        };
        let ret = unsafe {
            rmw_deserialize(
                &serialized,
                self.type_support.handle(),
                destination.as_mut_ptr().cast::<c_void>(),
            )
        };
        if ret != RMW_RET_OK as i32 {
            fail!(
                from origin,
                with TranscodeError::Transcoder(TranscodeFailure::Deserialize),
                "The rmw failed to deserialize a message of type '{}'", self.type_name
            );
        }

        Ok(())
    }
}

/// An `rcutils` allocator over a [`Region`], so the rmw serializes straight
/// into it.
fn region_allocator<R: Region>(region: &mut R) -> rcutils_allocator_t {
    unsafe extern "C" fn allocate<R: Region>(size: usize, state: *mut c_void) -> *mut c_void {
        let region = unsafe { &mut *(state as *mut R) };
        match region.for_length(size) {
            Ok(bytes) => bytes.as_mut_ptr().cast::<c_void>(),
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe extern "C" fn reallocate<R: Region>(
        _pointer: *mut c_void,
        size: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        // A region keeps its bytes when it grows, whatever pointer it hands
        // back.
        unsafe { allocate::<R>(size, state) }
    }

    unsafe extern "C" fn deallocate(_pointer: *mut c_void, _state: *mut c_void) {}

    unsafe extern "C" fn zero_allocate<R: Region>(
        number_of_elements: usize,
        size_of_element: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        let Some(size) = number_of_elements.checked_mul(size_of_element) else {
            return core::ptr::null_mut();
        };
        let region = unsafe { &mut *(state as *mut R) };
        let Ok(bytes) = region.for_length(size) else {
            return core::ptr::null_mut();
        };
        bytes.fill(0);
        bytes.as_mut_ptr().cast::<c_void>()
    }

    rcutils_allocator_t {
        allocate: Some(allocate::<R>),
        deallocate: Some(deallocate),
        reallocate: Some(reallocate::<R>),
        zero_allocate: Some(zero_allocate::<R>),
        state: (region as *mut R).cast::<c_void>(),
    }
}
