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
//! publisher.send(sample)?;
//!
//! # Ok(())
//! # }
//! ```

use crate::{
    port::publisher_impl::PublisherImpl,
    raw_sample::RawSampleMut,
    sample_mut::{internal::SampleMgmt, SampleMut, UninitializedSampleMut},
    service::{self, header::publish_subscribe::Header},
};
use iceoryx2_cal::shared_memory::*;
use std::{fmt::Debug, mem::MaybeUninit, sync::atomic::Ordering};

/// Acquired by a [`Publisher`] via [`Publisher::loan()`] or [`Publisher::loan_uninit()`]. It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
///
/// # Notes
///
/// Does not implement [`Send`] since it releases unsent samples in the [`Publisher`] and the
/// [`Publisher`] is not thread-safe!
///
/// The generic parameter `M` is either a `MessageType` or a [`core::mem::MaybeUninit<MessageType>`], depending
/// which API is used to obtain the sample.
#[derive(Debug)]
pub struct SampleMutImpl<'a, 'publisher, 'config, Service: service::Details<'config>, M: Debug> {
    pub(crate) publisher: &'publisher PublisherImpl<'a, 'config, Service, M>,
    ptr: RawSampleMut<Header, M>,
    offset_to_chunk: PointerOffset,
}

impl<'config, Service: service::Details<'config>, M: Debug> Drop
    for SampleMutImpl<'_, '_, 'config, Service, M>
{
    fn drop(&mut self) {
        self.publisher.release_sample(self.offset_to_chunk);
        self.publisher.loan_counter.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<'a, 'publisher, 'config, Service: service::Details<'config>, MessageType: Debug>
    SampleMutImpl<'a, 'publisher, 'config, Service, MaybeUninit<MessageType>>
{
    pub(crate) fn new(
        publisher: &'publisher PublisherImpl<'a, 'config, Service, MessageType>,
        ptr: RawSampleMut<Header, MaybeUninit<MessageType>>,
        offset_to_chunk: PointerOffset,
    ) -> Self {
        publisher.loan_counter.fetch_add(1, Ordering::Relaxed);

        // SAFETY: the transmute is not nice but safe since MaybeUninit is #[repr(transparent)} to the inner type
        let publisher = unsafe { std::mem::transmute(publisher) };

        Self {
            publisher,
            ptr,
            offset_to_chunk,
        }
    }
}

impl<'a, 'publisher, 'config, Service: service::Details<'config>, MessageType: Debug> SampleMgmt
    for SampleMutImpl<'a, 'publisher, 'config, Service, MessageType>
{
    fn originates_from(&self, publisher_address: usize) -> bool {
        publisher_address
            == (self.publisher as *const PublisherImpl<'a, 'config, Service, MessageType>) as usize
    }

    fn offset_to_chunk(&self) -> PointerOffset {
        self.offset_to_chunk
    }
}

impl<'a, 'publisher, 'config, Service: service::Details<'config>, MessageType: Debug>
    UninitializedSampleMut<MessageType>
    for SampleMutImpl<'a, 'publisher, 'config, Service, MaybeUninit<MessageType>>
{
    type InitializedSample = SampleMutImpl<'a, 'publisher, 'config, Service, MessageType>;

    fn write_payload(
        mut self,
        value: MessageType,
    ) -> SampleMutImpl<'a, 'publisher, 'config, Service, MessageType> {
        self.payload_mut().write(value);
        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }

    unsafe fn assume_init(self) -> SampleMutImpl<'a, 'publisher, 'config, Service, MessageType> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        std::mem::transmute(self)
    }
}

impl<
        'a,
        'publisher,
        'config,
        Service: service::Details<'config>,
        M: Debug, // `M` is either a `MessageType` or a `MaybeUninit<MessageType>`
    > SampleMut<M> for SampleMutImpl<'a, 'publisher, 'config, Service, M>
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
}
