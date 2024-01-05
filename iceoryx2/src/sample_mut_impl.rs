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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
//! #
//! # let service = zero_copy::Service::new(&service_name)
//! #     .publish_subscribe()
//! #     .open_or_create::<u64>()?;
//! #
//! # let publisher = service.publisher().create()?;
//!
//! let sample = publisher.loan_uninit()?;
//! let sample = sample.write_payload(1234);
//!
//! println!("timestamp: {:?}, publisher port id: {:?}",
//!     sample.header().time_stamp(), sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also, [`crate::sample_mut::SampleMut`].

use crate::{
    port::publisher::internal::PublisherMgmt,
    raw_sample::RawSampleMut,
    sample_mut::{internal::SampleMgmt, SampleMut, UninitializedSampleMut},
    service::header::publish_subscribe::Header,
};
use iceoryx2_cal::shared_memory::*;
use std::{fmt::Debug, mem::MaybeUninit};

/// Acquired by a [`crate::port::publisher::Publisher`] via
/// [`crate::port::publisher::PublisherLoan::loan()`] or
/// [`crate::port::publisher::Publisher::loan_uninit()`]. It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
///
/// # Notes
///
/// Does not implement [`Send`] since it releases unsent samples in the [`crate::port::publisher::Publisher`] and the
/// [`crate::port::publisher::Publisher`] is not thread-safe!
///
/// The generic parameter `M` is either a `MessageType` or a [`core::mem::MaybeUninit<MessageType>`], depending
/// which API is used to obtain the sample.
#[derive(Debug)]
pub struct SampleMutImpl<'publisher, M: Debug> {
    pub(crate) publisher: &'publisher dyn PublisherMgmt,
    ptr: RawSampleMut<Header, M>,
    offset_to_chunk: PointerOffset,
}

impl<M: Debug> Drop for SampleMutImpl<'_, M> {
    fn drop(&mut self) {
        self.publisher.return_loaned_sample(self.offset_to_chunk);
    }
}

impl<'publisher, MessageType: Debug> SampleMutImpl<'publisher, MaybeUninit<MessageType>> {
    pub(crate) fn new(
        publisher: &'publisher dyn PublisherMgmt,
        ptr: RawSampleMut<Header, MaybeUninit<MessageType>>,
        offset_to_chunk: PointerOffset,
    ) -> Self {
        // SAFETY: the transmute is not nice but safe since MaybeUninit is #[repr(transparent)} to the inner type
        let publisher = unsafe { std::mem::transmute(publisher) };

        Self {
            publisher,
            ptr,
            offset_to_chunk,
        }
    }
}

impl<'publisher, MessageType: Debug> SampleMgmt for SampleMutImpl<'publisher, MessageType> {
    fn offset_to_chunk(&self) -> PointerOffset {
        self.offset_to_chunk
    }
}

impl<'publisher, MessageType: Debug> UninitializedSampleMut<MessageType>
    for SampleMutImpl<'publisher, MaybeUninit<MessageType>>
{
    type InitializedSample = SampleMutImpl<'publisher, MessageType>;

    fn write_payload(mut self, value: MessageType) -> SampleMutImpl<'publisher, MessageType> {
        self.payload_mut().write(value);
        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }

    unsafe fn assume_init(self) -> SampleMutImpl<'publisher, MessageType> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        std::mem::transmute(self)
    }
}

impl<
        'publisher,
        M: Debug, // `M` is either a `MessageType` or a `MaybeUninit<MessageType>`
    > SampleMut<M> for SampleMutImpl<'publisher, M>
{
    fn header(&self) -> &Header {
        self.ptr.as_header_ref()
    }

    fn payload(&self) -> &M {
        self.ptr.as_data_ref()
    }

    fn payload_mut(&mut self) -> &mut M {
        self.ptr.as_data_mut()
    }

    fn send(self) -> Result<usize, iceoryx2_cal::zero_copy_connection::ZeroCopyCreationError> {
        self.publisher.send_impl(self.offset_to_chunk.value())
    }
}
