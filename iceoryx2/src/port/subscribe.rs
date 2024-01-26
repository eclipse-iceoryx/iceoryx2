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
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let pubsub_ipc = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let pubsub_local = process_local::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let mut subscribers: Vec<Box<dyn Subscriber<u64>>> = vec![];
//! subscribers.push(Box::new( pubsub_ipc.subscriber().create()?));
//! subscribers.push(Box::new( pubsub_local.subscriber().create()?));
//!
//! for subscriber in subscribers {
//!     while let Some(sample) = subscriber.receive()? {
//!         println!("received: {:?}", *sample);
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```

use std::fmt::Debug;

use crate::sample::Sample;

use super::details::publisher_connections::ConnectionFailure;

/// Defines the failure that can occur when receiving data with [`Subscriber::receive()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberReceiveError {
    ExceedsMaxBorrowedSamples,
    ConnectionFailure(ConnectionFailure),
}

impl std::fmt::Display for SubscriberReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for SubscriberReceiveError {}

/// Describes the failures when a new [`Subscriber`] is created via the
/// [`crate::service::port_factory::subscriber::PortFactorySubscriber`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberCreateError {
    ExceedsMaxSupportedSubscribers,
}

impl std::fmt::Display for SubscriberCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for SubscriberCreateError {}

pub(crate) mod internal {
    use std::fmt::Debug;

    pub(crate) trait SubscribeMgmt: Debug {
        fn release_sample(&self, channel_id: usize, sample: usize);
    }
}

/// The interface of the receiving endpoint of a publish-subscribe communication.
pub trait Subscribe<MessageType: Debug> {
    /// Receives a [`crate::sample::Sample`] from [`crate::port::publisher::Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`SubscriberReceiveError`] is returned.
    fn receive(&self) -> Result<Option<Sample<MessageType>>, SubscriberReceiveError>;

    /// Explicitly updates all connections to the [`crate::port::publisher::Publisher`]s. This is
    /// required to be called whenever a new [`crate::port::publisher::Publisher`] connected to
    /// the service. It is done implicitly whenever [`Subscriber::receive()`]
    /// is called.
    fn update_connections(&self) -> Result<(), ConnectionFailure>;
}
