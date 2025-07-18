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
//! ## Typed API
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let publisher = pubsub.publisher_builder()
//!                     .max_loaned_samples(6)
//!                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Slice API
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<[u64]>()
//!     .open_or_create()?;
//!
//! let publisher = pubsub.publisher_builder()
//!                     // allows to call Publisher::loan_slice() with up to 128 elements
//!                     .initial_max_slice_len(128)
//!                     .create()?;
//!
//! let sample = publisher.loan_slice(50)?;
//!
//! # Ok(())
//! # }
//! ```

use core::fmt::Debug;

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::shm_allocator::AllocationStrategy;

use super::publish_subscribe::PortFactory;
use crate::{
    port::{
        publisher::{Publisher, PublisherCreateError},
        unable_to_deliver_strategy::UnableToDeliverStrategy,
        DegradationAction, DegradationCallback,
    },
    service,
};

#[derive(Debug)]
pub(crate) struct LocalPublisherConfig {
    pub(crate) max_loaned_samples: usize,
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
    pub(crate) degradation_callback: Option<DegradationCallback<'static>>,
    pub(crate) initial_max_slice_len: usize,
    pub(crate) allocation_strategy: AllocationStrategy,
}

/// Factory to create a new [`Publisher`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) based
/// communication.
#[derive(Debug)]
pub struct PortFactoryPublisher<
    'factory,
    Service: service::Service,
    Payload: Debug + ZeroCopySend + ?Sized,
    UserHeader: Debug + ZeroCopySend,
> {
    config: LocalPublisherConfig,
    pub(crate) factory: &'factory PortFactory<Service, Payload, UserHeader>,
}

unsafe impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > Send for PortFactoryPublisher<'_, Service, Payload, UserHeader>
{
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > PortFactoryPublisher<'_, Service, Payload, UserHeader>
{
    #[doc(hidden)]
    /// # Safety
    ///
    ///   * does not clone the degradation callback
    pub unsafe fn __internal_partial_clone(&self) -> Self {
        Self {
            config: LocalPublisherConfig {
                max_loaned_samples: self.config.max_loaned_samples,
                unable_to_deliver_strategy: self.config.unable_to_deliver_strategy,
                degradation_callback: None,
                initial_max_slice_len: self.config.initial_max_slice_len,
                allocation_strategy: self.config.allocation_strategy,
            },
            factory: self.factory,
        }
    }
}

impl<
        'factory,
        Service: service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: Debug + ZeroCopySend,
    > PortFactoryPublisher<'factory, Service, Payload, UserHeader>
{
    pub(crate) fn new(factory: &'factory PortFactory<Service, Payload, UserHeader>) -> Self {
        Self {
            config: LocalPublisherConfig {
                allocation_strategy: AllocationStrategy::Static,
                degradation_callback: None,
                initial_max_slice_len: 1,
                max_loaned_samples: factory
                    .service
                    .shared_node
                    .config()
                    .defaults
                    .publish_subscribe
                    .publisher_max_loaned_samples,
                unable_to_deliver_strategy: factory
                    .service
                    .shared_node
                    .config()
                    .defaults
                    .publish_subscribe
                    .unable_to_deliver_strategy,
            },
            factory,
        }
    }

    /// Defines how many [`crate::sample_mut::SampleMut`] the [`Publisher`] can loan with
    /// [`Publisher::loan()`] or
    /// [`Publisher::loan_uninit()`] in parallel.
    pub fn max_loaned_samples(mut self, value: usize) -> Self {
        self.config.max_loaned_samples = value;
        self
    }

    /// Sets the [`UnableToDeliverStrategy`].
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.config.unable_to_deliver_strategy = value;
        self
    }

    /// Sets the [`DegradationCallback`] of the [`Publisher`]. Whenever a connection to a
    /// [`crate::port::subscriber::Subscriber`] is corrupted or it seems to be dead, this callback
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_degradation_callback<
        F: Fn(&service::static_config::StaticConfig, u128, u128) -> DegradationAction + 'static,
    >(
        mut self,
        callback: Option<F>,
    ) -> Self {
        match callback {
            Some(c) => self.config.degradation_callback = Some(DegradationCallback::new(c)),
            None => self.config.degradation_callback = None,
        }

        self
    }

    /// Creates a new [`Publisher`] or returns a [`PublisherCreateError`] on failure.
    pub fn create(self) -> Result<Publisher<Service, Payload, UserHeader>, PublisherCreateError> {
        let origin = format!("{self:?}");
        Ok(
            fail!(from origin, when Publisher::new(self.factory.service.clone(), self.factory.service.static_config.publish_subscribe(), self.config),
                "Failed to create new Publisher port."),
        )
    }
}

impl<
        Service: service::Service,
        Payload: Debug + ZeroCopySend,
        UserHeader: Debug + ZeroCopySend,
    > PortFactoryPublisher<'_, Service, [Payload], UserHeader>
{
    /// Sets the maximum slice length that a user can allocate with
    /// [`Publisher::loan_slice()`] or [`Publisher::loan_slice_uninit()`].
    pub fn initial_max_slice_len(mut self, value: usize) -> Self {
        self.config.initial_max_slice_len = value;
        self
    }

    /// Defines the allocation strategy that is used when the provided
    /// [`PortFactoryPublisher::initial_max_slice_len()`] is exhausted. This happens when the user
    /// acquires a more than max slice len in [`Publisher::loan_slice()`] or
    /// [`Publisher::loan_slice_uninit()`].
    pub fn allocation_strategy(mut self, value: AllocationStrategy) -> Self {
        self.config.allocation_strategy = value;
        self
    }
}
