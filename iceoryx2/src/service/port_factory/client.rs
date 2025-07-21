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
//!                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```

use super::request_response::PortFactory;
use crate::{
    port::{client::Client, DegradationAction, DegradationCallback},
    prelude::UnableToDeliverStrategy,
    service,
};
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::shm_allocator::AllocationStrategy;

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
    pub(crate) request_degradation_callback: Option<DegradationCallback<'static>>,
    pub(crate) response_degradation_callback: Option<DegradationCallback<'static>>,
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
    ///   * does not clone the degradation callback
    pub unsafe fn __internal_partial_clone(&self) -> Self {
        Self {
            config: self.config,
            request_degradation_callback: None,
            response_degradation_callback: None,
            factory: self.factory,
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
            request_degradation_callback: None,
            response_degradation_callback: None,
            factory,
        }
    }

    /// Sets the [`UnableToDeliverStrategy`] which defines how the [`Client`] shall behave
    /// when a [`Server`](crate::port::server::Server) cannot receive a
    /// [`RequestMut`](crate::request_mut::RequestMut) since
    /// its internal buffer is full.
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.config.unable_to_deliver_strategy = value;
        self
    }

    /// Sets the [`DegradationCallback`] for sending [`RequestMut`](crate::request_mut::RequestMut)
    /// from the [`Client`]. Whenever a connection to a
    /// [`Server`](crate::port::server::Server) is corrupted or it seems to be dead, this callback
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_request_degradation_callback<
        F: Fn(&service::static_config::StaticConfig, u128, u128) -> DegradationAction + 'static,
    >(
        mut self,
        callback: Option<F>,
    ) -> Self {
        match callback {
            Some(c) => self.request_degradation_callback = Some(DegradationCallback::new(c)),
            None => self.request_degradation_callback = None,
        }

        self
    }

    /// Sets the [`DegradationCallback`] for receiving [`Response`](crate::response::Response)s
    /// from a [`Server`](crate::port::server::Server). Whenever a connection to a
    /// [`Server`](crate::port::server::Server) is corrupted or it seems to be dead, this callback
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_response_degradation_callback<
        F: Fn(&service::static_config::StaticConfig, u128, u128) -> DegradationAction + 'static,
    >(
        mut self,
        callback: Option<F>,
    ) -> Self {
        match callback {
            Some(c) => self.response_degradation_callback = Some(DegradationCallback::new(c)),
            None => self.response_degradation_callback = None,
        }

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
    >
    PortFactoryClient<'_, Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
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
