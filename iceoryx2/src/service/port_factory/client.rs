// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let request_response = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! let client = request_response.client_builder()
//!                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardData)
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```

use super::request_response::PortFactory;
use crate::{
    port::{
        DegradationAction, DegradationFn, DegradationHandler, UnableToDeliverFn,
        UnableToDeliverHandler, client::Client,
    },
    prelude::UnableToDeliverStrategy,
    service,
};
use alloc::format;
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::shm_allocator::AllocationStrategy;
use iceoryx2_log::fail;
use tiny_fn::tiny_fn;

/// Defines a failure that can occur when a [`Client`] is created with
/// [`crate::service::port_factory::client::PortFactoryClient`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ClientCreateError {
    /// The datasegment in which the payload of the [`Client`] is stored, could not be created.
    UnableToCreateDataSegment,
    /// The maximum amount of [`Client`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Client`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedClients,
    /// Caused by a failure when instantiating a
    /// [`ArcSyncPolicy`](iceoryx2_cal::arc_sync_policy::ArcSyncPolicy) defined in the
    /// [`Service`](crate::service::Service) as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
}

impl core::fmt::Display for ClientCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ClientCreateError::{self:?}")
    }
}

impl core::error::Error for ClientCreateError {}

tiny_fn! {
    /// A user provided callback to reduce the number of preallocated [`RequestMut`](crate::request_mut::RequestMut)s.
    /// The input argument is the worst case number of preallocated [`RequestMut`](crate::request_mut::RequestMut)s required
    /// to guarantee that the [`Client`] never runs out of [`RequestMut`](crate::request_mut::RequestMut)s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`RequestMut`](crate::request_mut::RequestMut)s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`RequestMut`](crate::request_mut::RequestMut)s, iceoryx2 can
    /// no longer guarantee, that the [`Client`] can always loan a [`RequestMut`](crate::request_mut::RequestMut)
    /// to send.
    pub struct PreallocatedRequestsOverride = Fn(number_of_preallocated_requests: usize) -> usize;
}

impl<'a> Debug for PreallocatedRequestsOverride<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PreallocatedRequestsOverride")
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalClientConfig {
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
    pub(crate) initial_max_slice_len: usize,
    pub(crate) allocation_strategy: AllocationStrategy,
}

/// Factory to create a new [`Client`] port/endpoint for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based communication.
#[derive(Debug)]
pub struct PortFactoryClient<
    'factory,
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) config: LocalClientConfig,
    pub(crate) preallocate_number_of_requests_override: PreallocatedRequestsOverride<'static>,
    pub(crate) request_degradation_handler: DegradationHandler<'static>,
    pub(crate) response_degradation_handler: DegradationHandler<'static>,
    pub(crate) unable_to_deliver_handler: Option<UnableToDeliverHandler<'static>>,
    pub(crate) factory: &'factory PortFactory<
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >,
}

unsafe impl<
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Send
    for PortFactoryClient<
        '_,
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
}

impl<
    'factory,
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
>
    PortFactoryClient<
        'factory,
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    #[doc(hidden)]
    /// # Safety
    ///
    ///   * does not clone the callbacks
    pub unsafe fn __internal_partial_clone(&self) -> Self {
        Self {
            config: self.config,
            factory: self.factory,
            request_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            response_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            unable_to_deliver_handler: None,
            preallocate_number_of_requests_override: PreallocatedRequestsOverride::new(|v| v),
        }
    }

    pub(crate) fn new(
        factory: &'factory PortFactory<
            Service,
            RequestPayload,
            RequestHeader,
            ResponsePayload,
            ResponseHeader,
        >,
    ) -> Self {
        let defs = &factory
            .service
            .shared_node
            .config()
            .defaults
            .request_response;

        Self {
            config: LocalClientConfig {
                unable_to_deliver_strategy: defs.client_unable_to_deliver_strategy,
                initial_max_slice_len: 1,
                allocation_strategy: AllocationStrategy::Static,
            },
            preallocate_number_of_requests_override: PreallocatedRequestsOverride::new(|v| v),
            request_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            response_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            unable_to_deliver_handler: None,
            factory,
        }
    }

    /// Defines a callback to reduce the number of preallocated [`RequestMut`](crate::request_mut::RequestMut)s.
    /// The input argument is the worst case number of preallocated [`RequestMut`](crate::request_mut::RequestMut)s required
    /// to guarantee that the [`Client`] never runs out of [`RequestMut`](crate::request_mut::RequestMut)s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`RequestMut`](crate::request_mut::RequestMut)s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`RequestMut`](crate::request_mut::RequestMut)s, iceoryx2 can
    /// no longer guarantee, that the [`Client`] can always loan a [`RequestMut`](crate::request_mut::RequestMut)
    /// to send.
    pub fn override_request_preallocation<F: Fn(usize) -> usize + 'static>(
        mut self,
        callback: F,
    ) -> Self {
        self.preallocate_number_of_requests_override =
            PreallocatedRequestsOverride::new(move |v| callback(v).clamp(1, v));
        self
    }

    /// Sets the [`UnableToDeliverStrategy`] which defines how the [`Client`] shall behave
    /// when a [`Server`](crate::port::server::Server) cannot receive a
    /// [`RequestMut`](crate::request_mut::RequestMut) since
    /// its internal buffer is full.
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.config.unable_to_deliver_strategy = value;
        self
    }

    /// Sets the [`DegradationHandler`] for sending [`RequestMut`](crate::request_mut::RequestMut)
    /// from the [`Client`]. Whenever a request connection to a
    /// [`Server`](crate::port::server::Server) is corrupted or it seems to be dead, this handler
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_request_degradation_handler<F: DegradationFn + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.request_degradation_handler = DegradationHandler::new(handler);

        self
    }

    /// Sets the [`DegradationHandler`] for receiving [`Response`](crate::response::Response)s
    /// from a [`Server`](crate::port::server::Server). Whenever a response connection to a
    /// [`Server`](crate::port::server::Server) is corrupted or it seems to be dead, this handler
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_response_degradation_handler<F: DegradationFn + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.response_degradation_handler = DegradationHandler::new(handler);

        self
    }

    /// Sets the [`UnableToDeliverHandler`] of the [`Client`]. Whenever a
    /// [`RequestMut`](crate::response::Response) cannot be sent to a
    /// [`crate::port::server::Server`], this handler is called and depending on
    /// the returned [`UnableToDeliverAction`](crate::port::UnableToDeliverAction),
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

    /// Creates a new [`Client`] or returns a [`ClientCreateError`] on failure.
    pub fn create(
        self,
    ) -> Result<
        Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        ClientCreateError,
    > {
        let origin = format!("{self:?}");
        Ok(fail!(from origin,
              when Client::new(self),
              "Failed to create new Client port."))
    }
}

impl<
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> PortFactoryClient<'_, Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Sets the maximum slice length that a user can allocate with
    /// [`Client::loan_slice()`] or [`Client::loan_slice_uninit()`].
    pub fn initial_max_slice_len(mut self, value: usize) -> Self {
        self.config.initial_max_slice_len = value;
        self
    }

    /// Defines the allocation strategy that is used when the provided
    /// [`PortFactoryClient::initial_max_slice_len()`] is exhausted. This happens when the user
    /// acquires more than max slice len in [`Client::loan_slice()`] or
    /// [`Client::loan_slice_uninit()`].
    pub fn allocation_strategy(mut self, value: AllocationStrategy) -> Self {
        self.config.allocation_strategy = value;
        self
    }
}
