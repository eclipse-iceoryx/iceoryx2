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
//! let pubsub_local = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let mut publishers: Vec<Box<dyn Publisher<u64>>> = vec![];
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
//! See also, [`crate::port::publisher_impl::PublisherImpl`].

use std::{fmt::Debug, mem::MaybeUninit};

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_cal::zero_copy_connection::ZeroCopyCreationError;

use crate::sample_mut_impl::SampleMutImpl;

/// Defines a failure that can occur when a [`Publisher`] is created with
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

/// Defines a failure that can occur in [`Publisher::loan()`] and [`Publisher::loan_uninit()`] or is part of [`SendCopyError`]
/// emitted in [`Publisher::send_copy()`].
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
  entry:
    SampleDoesNotBelongToPublisher
  mapping:
    PublisherLoanError to LoanError,
    ZeroCopyCreationError to ConnectionError
}

impl std::fmt::Display for PublisherSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherSendError {}

enum_gen! {
    /// Failure that can be emitted when a [`crate::sample::Sample`] is sent via [`Publisher::send_copy()`].
    PublisherSendCopyError
  mapping:
    PublisherLoanError to LoanError,
    ZeroCopyCreationError to ConnectionError
}

impl std::fmt::Display for PublisherSendCopyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherSendCopyError {}

pub(crate) mod internal {
    use std::fmt::Debug;

    use iceoryx2_cal::zero_copy_connection::PointerOffset;

    pub(crate) trait PublisherMgmt: Debug {
        fn return_loaned_sample(&self, distance_to_chunk: PointerOffset);
        fn publisher_address(&self) -> usize;
    }
}

/// Interface of the sending endpoint of a publish-subscriber based communication.
pub trait Publisher<MessageType: Debug> {
    /// Explicitly updates all connections to the [`crate::port::subscriber::Subscriber`]s. This is
    /// required to be called whenever a new [`crate::port::subscriber::Subscriber`] connected to
    /// the service. It is done implicitly whenever [`Publisher::send()`] or [`Publisher::send_copy()`]
    /// is called.
    fn update_connections(&self) -> Result<(), ZeroCopyCreationError>;

    /// Send a previously loaned [`Publisher::loan_uninit()`] [`SampleMut`] to all connected
    /// [`crate::port::subscriber::Subscriber`]s of the service.
    ///
    /// The payload of the [`SampleMut`] must be initialized before it can be sent. Have a look
    /// at [`SampleMut::write_payload()`] and [`SampleMut::assume_init()`] for more details.
    ///
    /// On success the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data is returned, otherwise a [`ZeroCopyCreationError`] describing the failure.
    fn send<'publisher>(
        &'publisher self,
        sample: SampleMutImpl<'publisher, MessageType>,
    ) -> Result<usize, PublisherSendError>;

    /// Copies the input `value` into a [`SampleMut`] and delivers it.
    /// On success it returns the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data, otherwise a [`SendCopyError`] describing the failure.
    fn send_copy(&self, value: MessageType) -> Result<usize, PublisherSendCopyError>;

    /// Loans/allocates a [`SampleMut`] from the underlying data segment of the [`Publisher`].
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// Example
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
    /// publisher.send(sample)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn loan_uninit<'publisher>(
        &'publisher self,
    ) -> Result<SampleMutImpl<'publisher, MaybeUninit<MessageType>>, PublisherLoanError>;
}

/// Interface of the sending endpoint of a publish-subscriber based communication that
/// provides a `PublisherLoan::loan()` to create default initialized samples.
pub trait PublisherLoan<MessageType: Debug + Default>: Publisher<MessageType> {
    /// Loans/allocates a [`SampleMut`] from the underlying data segment of the [`Publisher`]
    /// and initialize it with the default value. This can be a performance hit and [`Publisher::loan_uninit`]
    /// can be used to loan a [`core::mem::MaybeUninit<MessageType>`].
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// Example
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
    /// publisher.send(sample)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn loan<'publisher>(
        &'publisher self,
    ) -> Result<SampleMutImpl<'publisher, MessageType>, PublisherLoanError>;
}
