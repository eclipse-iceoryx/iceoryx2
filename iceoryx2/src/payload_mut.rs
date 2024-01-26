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
//! ## Sample Write Function
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! fn write_sample_data<S: UninitPayloadMut<u64>>(mut sample: S) -> impl PayloadMut<u64> {
//!     sample.write_payload(123456)
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
//! #
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let publisher = service.publisher().create()?;
//!
//! let sample = publisher.loan_uninit()?;
//! let sample = write_sample_data(sample);
//!
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```
//! See also, [`crate::sample_mut::SampleMut`].

use crate::{
    port::update_connections::ConnectionFailure, service::header::publish_subscribe::Header,
};

pub(crate) mod internal {
    use iceoryx2_cal::zero_copy_connection::PointerOffset;

    pub trait PayloadMgmt {
        fn offset_to_chunk(&self) -> PointerOffset;
    }
}

/// Acquired by a [`crate::port::publisher::Publisher`] via
/// [`crate::port::publish::DefaultLoan::loan()`]. It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`PayloadMut`] is not sent
/// it will release the loaned memory when going out of scope.
pub trait PayloadMut<MessageType>: internal::PayloadMgmt {
    /// Returns a reference to the header of the sample.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// # let publisher = service.publisher().create()?;
    ///
    /// let sample = publisher.loan()?;
    /// println!("Sample Publisher Origin {:?}", sample.header().publisher_id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn header(&self) -> &Header;

    /// Returns a reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `MessageType` can be packed into [`core::mem::MaybeUninit<MessageType>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// # let publisher = service.publisher().create()?;
    ///
    /// let sample = publisher.loan()?;
    /// println!("Sample current payload {}", sample.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn payload(&self) -> &MessageType;

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `MessageType` can be packed into [`core::mem::MaybeUninit<MessageType>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .open_or_create::<u64>()?;
    /// # let publisher = service.publisher().create()?;
    ///
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 4567;
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn payload_mut(&mut self) -> &mut MessageType;

    /// Send a previously loaned [`crate::port::publish::UninitLoan::loan_uninit()`] or
    /// [`crate::port::publish::DefaultLoan::loan()`] [`PayloadMut`] to all connected
    /// [`crate::port::subscriber::Subscriber`]s of the service.
    ///
    /// The payload of the [`PayloadMut`] must be initialized before it can be sent. Have a look
    /// at [`UninitPayloadMut::write_payload()`] and [`UninitPayloadMut::assume_init()`]
    /// for more details.
    ///
    /// On success the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data is returned, otherwise a [`ConnectionFailure`] describing the failure.
    fn send(self) -> Result<usize, ConnectionFailure>;
}

/// Acquired by a [`crate::port::publisher::Publisher`] via
/// [`crate::port::publish::UninitLoan::loan_uninit()`]. It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`PayloadMut`] is not sent
/// it will release the loaned memory when going out of scope.
///
/// The generic parameter `MessageType` is packed into [`core::mem::MaybeUninit<MessageType>`]
pub trait UninitPayloadMut<MessageType> {
    type InitializedSample: PayloadMut<MessageType>;

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
    /// sample.send()?;
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
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    unsafe fn assume_init(self) -> Self::InitializedSample;
}
