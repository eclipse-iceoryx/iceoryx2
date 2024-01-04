// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
//! use iceoryx2::sample_mut_impl::SampleMutImpl;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
//! #
//! # let service = zero_copy::Service::new(&service_name)
//! #     .publish_subscribe()
//! #     .open_or_create::<u64>()?;
//! #
//! # let publisher = service.publisher().create()?;
//!
//! let samples: Vec<Box<dyn SampleMut<u64>>> = vec![];
//!
//! let sample_1 = publisher.loan_uninit()?;
//! let sample_1 = sample_1.write_payload(1234);
//!
//! let sample_2 = publisher.loan_uninit()?;
//! let sample_2 = sample_2.write_payload(456);
//!
//! samples.push(Box::new(sample_1));
//! samples.push(Box::new(sample_2));
//!
//! publisher.send(*samples.pop().unwrap())?;
//! publisher.send(*samples.pop().unwrap())?;
//!
//! # Ok(())
//! # }
//! ```

use crate::service::header::publish_subscribe::Header;

/// Acquired by a [`Publisher`] via [`Publisher::loan()`]. It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
///
/// # Notes
///
/// Does not implement [`Send`] since it releases unsent samples in the [`Publisher`] and the
/// [`Publisher`] is not thread-safe!
pub trait SampleMut<MessageType> {
    /// Returns a reference to the header of the sample.
    fn header(&self) -> &Header;

    /// Returns a reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `MessageType` can be packed into [`core::mem::MaybeUninit<MessageType>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    fn payload(&self) -> &MessageType;

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `MessageType` can be packed into [`core::mem::MaybeUninit<MessageType>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    fn payload_mut(&mut self) -> &mut MessageType;
}

/// Acquired by a [`Publisher`] via [`Publisher::loan_uninit()`]. It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
///
/// The generic parameter `MessageType` is packed into [`core::mem::MaybeUninit<MessageType>`]
pub trait UninitializedSampleMut<MessageType> {
    type InitializedSample;

    /// Writes the payload to the sample and labels the sample as initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// #
    /// # let publisher = service.publisher().create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// let sample = sample.write_payload(1234);
    ///
    /// publisher.send(sample)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn write_payload(self, value: MessageType) -> Self::InitializedSample;

    /// Extracts the value of the [`core::mem::MaybeUninit<MessageType>`] container and labels the sample as initialized
    ///
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<MessageType>`] really is initialized. Calling this when
    /// the content is not fully initialized causes immediate undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// #
    /// # let publisher = service.publisher().create()?;
    ///
    /// let mut sample = publisher.loan_uninit()?;
    /// sample.payload_mut().write(1234);
    /// let sample = unsafe { sample.assume_init() };
    ///
    /// publisher.send(sample)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    unsafe fn assume_init(self) -> Self::InitializedSample;
}
