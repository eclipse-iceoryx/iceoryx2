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
//!                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardData)
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

use crate::{
    port::{
        DegradationAction, DegradationFn, DegradationHandler, UnableToDeliverFn,
        UnableToDeliverHandler,
        publisher::{Publisher, PublisherCreateError},
        unable_to_deliver_strategy::UnableToDeliverStrategy,
    },
    service,
};
use alloc::format;
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::shm_allocator::AllocationStrategy;
use iceoryx2_log::fail;
use tiny_fn::tiny_fn;

use super::publish_subscribe::PortFactory;

tiny_fn! {
    /// A user provided callback to reduce the number of preallocated [`SampleMut`](crate::sample_mut::SampleMut)s.
    /// The input argument is the worst case number of preallocated [`SampleMut`](crate::sample_mut::SampleMut)s required
    /// to guarantee that the [`Publisher`] never runs out of [`SampleMut`](crate::sample_mut::SampleMut)s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`SampleMut`](crate::sample_mut::SampleMut)s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`SampleMut`](crate::sample_mut::SampleMut)s, iceoryx2 can
    /// no longer guarantee, that the [`Publisher`] can always loan a [`SampleMut`](crate::sample_mut::SampleMut)
    /// to send.
    pub struct PreallocatedSamplesOverride = Fn(number_of_preallocated_samples: usize) -> usize;
}

impl<'a> Debug for PreallocatedSamplesOverride<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PreallocatedSamplesOverride")
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalPublisherConfig {
    pub(crate) max_loaned_samples: usize,
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
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
    pub(crate) config: LocalPublisherConfig,
    pub(crate) degradation_handler: DegradationHandler<'static>,
    pub(crate) unable_to_deliver_handler: Option<UnableToDeliverHandler<'static>>,
    pub(crate) preallocate_number_of_samples_override: PreallocatedSamplesOverride<'static>,
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
    ///   * does not clone the callbacks
    pub unsafe fn __internal_partial_clone(&self) -> Self {
        Self {
            config: self.config,
            factory: self.factory,
            degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            unable_to_deliver_handler: None,
            preallocate_number_of_samples_override: PreallocatedSamplesOverride::new(|v| v),
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
            degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            unable_to_deliver_handler: None,
            preallocate_number_of_samples_override: PreallocatedSamplesOverride::new(|v| v),
            factory,
        }
    }

    /// Defines a callback to reduce the number of preallocated [`SampleMut`](crate::sample_mut::SampleMut).
    /// The input argument is the worst case number of preallocated [`SampleMut`](crate::sample_mut::SampleMut)s required
    /// to guarantee that the [`Publisher`] never runs out of [`SampleMut`](crate::sample_mut::SampleMut)s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`SampleMut`](crate::sample_mut::SampleMut)s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`SampleMut`](crate::sample_mut::SampleMut)s, iceoryx2 can
    /// no longer guarantee, that the [`Publisher`] can always loan a [`SampleMut`](crate::sample_mut::SampleMut)
    /// to send.
    pub fn override_sample_preallocation<F: Fn(usize) -> usize + 'static>(
        mut self,
        callback: F,
    ) -> Self {
        self.preallocate_number_of_samples_override =
            PreallocatedSamplesOverride::new(move |v| callback(v).clamp(1, v));
        self
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

    /// Sets the [`DegradationHandler`] of the [`Publisher`]. Whenever a connection to a
    /// [`crate::port::subscriber::Subscriber`] is corrupted or it seems to be dead, this handler
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_degradation_handler<F: DegradationFn + 'static>(mut self, handler: F) -> Self {
        self.degradation_handler = DegradationHandler::new(handler);

        self
    }

    /// Sets the [`UnableToDeliverHandler`] of the [`Publisher`]. Whenever a
    /// [`SampleMut`](crate::sample_mut::SampleMut) cannot be sent to a
    /// [`crate::port::subscriber::Subscriber`], this handler is called and depending on the
    /// returned [`UnableToDeliverAction`](crate::port::UnableToDeliverAction),
    /// measures will be taken.
    /// If no handler is set, the measures will be determined by the value set in
    /// [`UnableToDeliverStrategy`].
    pub fn set_unable_to_deliver_handler<F: UnableToDeliverFn + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.unable_to_deliver_handler = Some(UnableToDeliverHandler::new(handler));

        self
    }

    /// Creates a new [`Publisher`] or returns a [`PublisherCreateError`] on failure.
    pub fn create(self) -> Result<Publisher<Service, Payload, UserHeader>, PublisherCreateError> {
        let origin = format!("{self:?}");
        Ok(fail!(from origin, when Publisher::new(self),
                "Failed to create new Publisher port."))
    }
}

impl<Service: service::Service, Payload: Debug + ZeroCopySend, UserHeader: Debug + ZeroCopySend>
    PortFactoryPublisher<'_, Service, [Payload], UserHeader>
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
