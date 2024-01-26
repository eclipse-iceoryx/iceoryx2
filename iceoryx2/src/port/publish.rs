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
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
//! #
//! let pubsub_ipc = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let pubsub_local = process_local::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let mut publishers: Vec<Box<dyn Publish<u64>>> = vec![];
//!
//! publishers.push(Box::new(pubsub_ipc.publisher().create()?));
//! publishers.push(Box::new(pubsub_local.publisher().create()?));
//!
//! for publisher in publishers {
//!     publisher.send_copy(1234);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also, [`crate::port::publisher::Publisher`].

use crate::port::update_connections::UpdateConnections;
use std::{fmt::Debug, mem::MaybeUninit};

use iceoryx2_bb_elementary::enum_gen;

use crate::sample_mut::SampleMut;

use super::update_connections::ConnectionFailure;

/// Defines a failure that can occur when a [`Publish`] is created with
/// [`crate::service::port_factory::publisher::PortFactoryPublisher`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PublisherCreateError {
    ExceedsMaxSupportedPublishers,
    UnableToCreateDataSegment,
}

impl std::fmt::Display for PublisherCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherCreateError {}

/// Defines a failure that can occur in [`DefaultLoan::loan()`] and [`UninitLoan::loan_uninit()`]
/// or is part of [`PublisherSendError`] emitted in [`SendCopy::send_copy()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PublisherLoanError {
    OutOfMemory,
    ExceedsMaxLoanedChunks,
    InternalFailure,
}

impl std::fmt::Display for PublisherLoanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherLoanError {}

enum_gen! {
    /// Failure that can be emitted when a [`crate::sample::Sample`] is sent via [`Publisher::send()`].
    PublisherSendError
  mapping:
    PublisherLoanError to LoanError,
    ConnectionFailure to ConnectionError
}

impl std::fmt::Display for PublisherSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherSendError {}

pub(crate) mod internal {
    use std::fmt::Debug;

    use iceoryx2_cal::zero_copy_connection::PointerOffset;

    use crate::port::update_connections::ConnectionFailure;

    pub(crate) trait PublishMgmt: Debug {
        fn return_loaned_sample(&self, distance_to_chunk: PointerOffset);
        fn send_impl(&self, address_to_chunk: usize) -> Result<usize, ConnectionFailure>;
    }
}

/// Interface of the sending endpoint of a publish-subscriber based communication.
pub trait Publish<MessageType: Debug + Default>:
    DefaultLoan<MessageType> + UninitLoan<MessageType> + UpdateConnections + SendCopy<MessageType>
{
}

/// Copies the payload into the shared memory and sends it.
pub trait SendCopy<MessageType: Debug> {
    /// Copies the input `value` into a [`crate::sample_mut::SampleMut`] and delivers it.
    /// On success it returns the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data, otherwise a [`PublisherSendError`] describing the failure.
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
    /// publisher.send_copy(1234)?;
    /// # Ok(())
    /// # }
    /// ```
    fn send_copy(&self, value: MessageType) -> Result<usize, PublisherSendError>;
}

/// Allows loaning of uninitialized shared memory that can be used for storing the payload of the message.
pub trait UninitLoan<MessageType: Debug> {
    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publish`]er.
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
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
    /// let sample = sample.write_payload(42); // alternatively `sample.payload_mut()` can be use to access the `MaybeUninit<MessageType>`
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn loan_uninit(&self) -> Result<SampleMut<MaybeUninit<MessageType>>, PublisherLoanError>;
}

/// Allows loaning shared memory that can be used for storing the payload of the message.
pub trait DefaultLoan<MessageType: Debug + Default> {
    /// Loans/allocates a [`crate::sample_mut::SampleMut`] from the underlying data segment of the [`Publish`]
    /// and initialize it with the default value. This can be a performance hit and [`UninitLoan::loan_uninit`]
    /// can be used to loan a [`core::mem::MaybeUninit<MessageType>`].
    ///
    /// On failure it returns [`PublisherLoanError`] describing the failure.
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
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 42;
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn loan(&self) -> Result<SampleMut<MessageType>, PublisherLoanError>;
}
