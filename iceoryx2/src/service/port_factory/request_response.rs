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
//! let req_res = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! println!("name: {:?}", req_res.name());
//! println!("service id: {:?}", req_res.service_id());
//! println!("request type details: {:?}", req_res.static_config().request_message_type_details());
//! println!("response type details: {:?}", req_res.static_config().response_message_type_details());
//! println!("max active requests per client: {:?}", req_res.static_config().max_active_requests_per_client());
//! println!("max response buffer size: {:?}", req_res.static_config().max_response_buffer_size());
//! println!("max borrowed responses per pending responses: {:?}", req_res.static_config().max_borrowed_responses_per_pending_response());
//! println!("max servers: {:?}", req_res.static_config().max_clients());
//! println!("max clients: {:?}", req_res.static_config().max_servers());
//! println!("max nodes: {:?}", req_res.static_config().max_nodes());
//! println!("request safe overflow: {:?}", req_res.static_config().has_safe_overflow_for_requests());
//! println!("response safe overflow: {:?}", req_res.static_config().has_safe_overflow_for_responses());
//!
//! # Ok(())
//! # }
//! ```

use core::{fmt::Debug, marker::PhantomData};

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::{
    node::NodeListFailure,
    prelude::AttributeSet,
    service::{
        self, dynamic_config, service_id::ServiceId, service_name::ServiceName, static_config,
        NoResource, ServiceState,
    },
};

use super::{client::PortFactoryClient, nodes, server::PortFactoryServer};

extern crate alloc;
use alloc::sync::Arc;

/// The factory for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse).
/// It can acquire dynamic and static service informations and create
/// [`crate::port::client::Client`]
/// or [`crate::port::server::Server`] ports.
#[derive(Debug)]
pub struct PortFactory<
    Service: service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) service: Arc<ServiceState<Service, NoResource>>,
    _request_payload: PhantomData<RequestPayload>,
    _request_header: PhantomData<RequestHeader>,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

unsafe impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Send
    for PortFactory<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
}

unsafe impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Sync
    for PortFactory<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Clone
    for PortFactory<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > crate::service::port_factory::PortFactory
    for PortFactory<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    type Service = Service;
    type StaticConfig = static_config::request_response::StaticConfig;
    type DynamicConfig = dynamic_config::request_response::DynamicConfig;

    fn name(&self) -> &ServiceName {
        self.service.static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.static_config.attributes()
    }

    fn static_config(&self) -> &Self::StaticConfig {
        self.service.static_config.request_response()
    }

    fn dynamic_config(&self) -> &Self::DynamicConfig {
        self.service.dynamic_storage.get().request_response()
    }

    fn nodes<F: FnMut(crate::node::NodeState<Service>) -> CallbackProgression>(
        &self,
        callback: F,
    ) -> Result<(), NodeListFailure> {
        nodes(
            self.service.dynamic_storage.get(),
            self.service.shared_node.config(),
            callback,
        )
    }
}

impl<
        Service: service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > PortFactory<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub(crate) fn new(service: ServiceState<Service, NoResource>) -> Self {
        Self {
            service: Arc::new(service),
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }

    /// Returns a [`PortFactoryClient`] to create a new
    /// [`crate::port::client::Client`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// let request_response = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .request_response::<u64, u64>()
    ///     .open_or_create()?;
    ///
    /// let client = request_response
    ///         .client_builder()
    ///         .create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn client_builder(
        &self,
    ) -> PortFactoryClient<
        '_,
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    > {
        PortFactoryClient::new(self)
    }

    /// Returns a [`PortFactoryServer`] to create a new
    /// [`crate::port::server::Server`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// let request_response = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    ///     .request_response::<u64, u64>()
    ///     .open_or_create()?;
    ///
    /// let client = request_response
    ///         .server_builder()
    ///         .create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn server_builder(
        &self,
    ) -> PortFactoryServer<
        '_,
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    > {
        PortFactoryServer::new(self)
    }
}
