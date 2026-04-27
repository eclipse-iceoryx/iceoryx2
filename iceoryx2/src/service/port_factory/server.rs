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
//! ## Typed API
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
//! let server = request_response
//!    .server_builder()
//!    // defines behavior when client queue is full in a non-overflowing service
//!    .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardData)
//!    .create()?;
//!
//! # Ok(())
//! # }
//! ```

use super::request_response::PortFactory;
use crate::{
    port::{
        DegradationAction, DegradationFn, DegradationHandler, UnableToDeliverFn,
        UnableToDeliverHandler, server::Server,
    },
    prelude::UnableToDeliverStrategy,
    service,
};
use alloc::format;
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::shm_allocator::AllocationStrategy;
use iceoryx2_log::{fail, warn};
use tiny_fn::tiny_fn;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalServerConfig {
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
    pub(crate) initial_max_slice_len: usize,
    pub(crate) allocation_strategy: AllocationStrategy,
    pub(crate) max_loaned_responses_per_request: usize,
}

/// Defines a failure that can occur when a [`Server`] is created with
/// [`crate::service::port_factory::server::PortFactoryServer`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ServerCreateError {
    /// The maximum amount of [`Server`]s supported by the [`Service`](crate::service::Service)
    /// is already connected.
    ExceedsMaxSupportedServers,
    /// The datasegment in which the payload of the [`Server`] is stored, could not be created.
    UnableToCreateDataSegment,
    /// Caused by a failure when instantiating a
    /// [`ArcSyncPolicy`](iceoryx2_cal::arc_sync_policy::ArcSyncPolicy) defined in the
    /// [`Service`](crate::service::Service) as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
}

impl core::fmt::Display for ServerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ServerCreateError::{self:?}")
    }
}

impl core::error::Error for ServerCreateError {}

tiny_fn! {
    /// A user provided callback to reduce the number of preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s.
    /// The input argument is the worst case number of preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s required
    /// to guarantee that the [`Server`]/[`ActiveRequest`](crate::active_request::ActiveRequest) never runs out of [`ResponseMut`](crate::response_mut::ResponseMut)s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s, iceoryx2 can
    /// no longer guarantee, that the [`Client`] can always loan a [`ResponseMut`](crate::response_mut::ResponseMut)
    /// to send.
    pub struct PreallocatedResponseOverride = Fn(number_of_preallocated_responses: usize) -> usize;
}

impl<'a> Debug for PreallocatedResponseOverride<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PreallocatedResponseOverride")
    }
}

/// Factory to create a new [`Server`] port/endpoint for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based communication.
#[derive(Debug)]
pub struct PortFactoryServer<
    'factory,
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) factory: &'factory PortFactory<
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >,

    pub(crate) config: LocalServerConfig,
    pub(crate) request_degradation_handler: DegradationHandler<'static>,
    pub(crate) response_degradation_handler: DegradationHandler<'static>,
    pub(crate) unable_to_deliver_handler: Option<UnableToDeliverHandler<'static>>,
    pub(crate) preallocated_number_of_responses_override: PreallocatedResponseOverride<'static>,
}

unsafe impl<
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Send
    for PortFactoryServer<
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
    PortFactoryServer<
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
    ///   * does not clone the degradation callback
    pub unsafe fn __internal_partial_clone(&self) -> Self {
        Self {
            factory: self.factory,
            config: self.config,
            request_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            response_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            unable_to_deliver_handler: None,
            preallocated_number_of_responses_override: PreallocatedResponseOverride::new(|v| v),
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
            factory,
            config: LocalServerConfig {
                unable_to_deliver_strategy: defs.server_unable_to_deliver_strategy,
                initial_max_slice_len: 1,
                allocation_strategy: AllocationStrategy::Static,
                max_loaned_responses_per_request: defs.server_max_loaned_responses_per_request,
            },
            request_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            response_degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
            unable_to_deliver_handler: None,
            preallocated_number_of_responses_override: PreallocatedResponseOverride::new(|v| v),
        }
    }

    /// A user provided callback to reduce the number of preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s.
    /// The input argument is the worst case number of preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s required
    /// to guarantee that the [`Server`]/[`ActiveRequest`](crate::active_request::ActiveRequest) never runs out of [`ResponseMut`](crate::response_mut::ResponseMut)s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`ResponseMut`](crate::response_mut::ResponseMut)s, iceoryx2 can
    /// no longer guarantee, that the [`Server`] can always loan a [`ResponseMut`](crate::response_mut::ResponseMut)
    /// to send.
    pub fn override_response_preallocation<F: Fn(usize) -> usize + 'static>(
        mut self,
        callback: F,
    ) -> Self {
        self.preallocated_number_of_responses_override =
            PreallocatedResponseOverride::new(move |v| callback(v).clamp(1, v));
        self
    }

    /// Sets the [`UnableToDeliverStrategy`] which defines how the [`Server`] shall behave
    /// when a [`Client`](crate::port::client::Client) cannot receive a
    /// [`Response`](crate::response::Response) since
    /// its internal buffer is full.
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.config.unable_to_deliver_strategy = value;
        self
    }

    /// Defines the maximum number of [`ResponseMut`](crate::response_mut::ResponseMut) that
    /// the [`Server`] can loan in parallel per
    /// [`ActiveRequest`](crate::active_request::ActiveRequest).
    pub fn max_loaned_responses_per_request(mut self, value: usize) -> Self {
        if value == 0 {
            warn!(from self,
                "A value of 0 is not allowed for max loaned responses per request. Adjusting it to 1.");
        }
        self.config.max_loaned_responses_per_request = value.max(1);
        self
    }

    /// Sets the [`DegradationHandler`] for receiving [`ActiveRequest`](crate::active_request::ActiveRequest)s
    /// from a [`Client`](crate::port::client::Client). Whenever a request connection to a
    /// [`Client`](crate::port::client::Client) is corrupted or it seems to be dead, this handler
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_request_degradation_handler<F: DegradationFn + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.request_degradation_handler = DegradationHandler::new(handler);

        self
    }

    /// Sets the [`DegradationHandler`] for sending
    /// [`ResponseMut`](crate::response_mut::ResponseMut)s
    /// to a [`Client`](crate::port::client::Client). Whenever a response connection to a
    /// [`Client`](crate::port::client::Client) is corrupted or it seems to be dead, this handler
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_response_degradation_handler<F: DegradationFn + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.response_degradation_handler = DegradationHandler::new(handler);

        self
    }

    /// Sets the [`UnableToDeliverHandler`] of the [`Server`]. Whenever a
    /// [`ResponseMut`](crate::response_mut::ResponseMut) cannot be sent to a
    /// [`crate::port::client::Client`], this handler is called and depending on
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

    /// Creates a new [`Server`] or returns a [`ServerCreateError`] on failure.
    pub fn create(
        self,
    ) -> Result<
        Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        ServerCreateError,
    > {
        let origin = format!("{self:?}");
        Ok(fail!(from origin,
              when Server::new(self),
              "Failed to create new Server port."))
    }
}

impl<
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> PortFactoryServer<'_, Service, RequestPayload, RequestHeader, [ResponsePayload], ResponseHeader>
{
    /// Sets the maximum slice length that a user can allocate with
    /// [`ActiveRequest::loan_slice()`](crate::active_request::ActiveRequest::loan_slice()) or
    /// [`ActiveRequest::loan_slice_uninit()`](crate::active_request::ActiveRequest::loan_slice_uninit()).
    pub fn initial_max_slice_len(mut self, value: usize) -> Self {
        self.config.initial_max_slice_len = value;
        self
    }

    /// Defines the allocation strategy that is used when the provided
    /// [`PortFactoryServer::initial_max_slice_len()`] is exhausted. This happens when the user
    /// acquires more than max slice len in
    /// [`ActiveRequest::loan_slice()`](crate::active_request::ActiveRequest::loan_slice()) or
    /// [`ActiveRequest::loan_slice_uninit()`](crate::active_request::ActiveRequest::loan_slice_uninit()).
    pub fn allocation_strategy(mut self, value: AllocationStrategy) -> Self {
        self.config.allocation_strategy = value;
        self
    }
}
