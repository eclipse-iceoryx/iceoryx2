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
//! println!("request type details: {:?}", req_res.static_config().request_message_type_details());
//! println!("response type details: {:?}", req_res.static_config().response_message_type_details());
//! println!("max active requests per client: {:?}", req_res.static_config().max_active_requests_per_client());
//! println!("max response buffer size: {:?}", req_res.static_config().max_response_buffer_size());
//! println!("client max loaned requests: {:?}", req_res.static_config().max_loaned_requests());
//! println!("max servers: {:?}", req_res.static_config().max_clients());
//! println!("max clients: {:?}", req_res.static_config().max_servers());
//! println!("max nodes: {:?}", req_res.static_config().max_nodes());
//! println!("request safe overflow: {:?}", req_res.static_config().has_safe_overflow_for_requests());
//! println!("response safe overflow: {:?}", req_res.static_config().has_safe_overflow_for_responses());
//! println!("max borrowed responses per pending response: {:?}", req_res.static_config().max_borrowed_responses_per_pending_response());
//! println!("does support fire and forget requests: {:?}", req_res.static_config().does_support_fire_and_forget_requests());
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use serde::{Deserialize, Serialize};

use crate::config;

use super::message_type_details::MessageTypeDetails;

/// The static configuration of an
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`](crate::service::Service).
#[derive(Debug, Clone, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct StaticConfig {
    pub(crate) enable_safe_overflow_for_requests: bool,
    pub(crate) enable_safe_overflow_for_responses: bool,
    pub(crate) enable_fire_and_forget_requests: bool,
    pub(crate) max_active_requests_per_client: usize,
    pub(crate) max_loaned_requests: usize,
    pub(crate) max_response_buffer_size: usize,
    pub(crate) max_servers: usize,
    pub(crate) max_clients: usize,
    pub(crate) max_nodes: usize,
    pub(crate) max_borrowed_responses_per_pending_response: usize,
    pub(crate) request_message_type_details: MessageTypeDetails,
    pub(crate) response_message_type_details: MessageTypeDetails,
}

impl StaticConfig {
    pub(crate) fn new(config: &config::Config) -> Self {
        Self {
            enable_safe_overflow_for_requests: config
                .defaults
                .request_response
                .enable_safe_overflow_for_requests,
            enable_safe_overflow_for_responses: config
                .defaults
                .request_response
                .enable_safe_overflow_for_responses,
            max_active_requests_per_client: config
                .defaults
                .request_response
                .max_active_requests_per_client,
            max_response_buffer_size: config.defaults.request_response.max_response_buffer_size,
            max_servers: config.defaults.request_response.max_servers,
            max_clients: config.defaults.request_response.max_clients,
            max_nodes: config.defaults.request_response.max_nodes,
            max_borrowed_responses_per_pending_response: config
                .defaults
                .request_response
                .max_borrowed_responses_per_pending_response,
            max_loaned_requests: config.defaults.request_response.max_loaned_requests,
            enable_fire_and_forget_requests: config
                .defaults
                .request_response
                .enable_fire_and_forget_requests,
            request_message_type_details: MessageTypeDetails::default(),
            response_message_type_details: MessageTypeDetails::default(),
        }
    }

    pub(crate) fn required_amount_of_chunks_per_client_data_segment(
        &self,
        client_max_loaned_data: usize,
    ) -> usize {
        // all chunks a server can hold
        self.max_servers * (
            // a client sent so many active requests to a server in parallel
            self.max_active_requests_per_client +
            // the server can still hold old requests that the client has already dropped. in this case
            // the client can fill up the server's buffer with at most max_active_requests_per_client again
            self.max_active_requests_per_client
        )
        // all chunks a client can loan in parallel
            + client_max_loaned_data
    }

    pub(crate) fn required_amount_of_chunks_per_server_data_segment(
        &self,
        max_loaned_responses_per_request: usize,
        total_number_of_requests_per_client: usize,
    ) -> usize {
        let total_number_of_requests = self.max_clients * total_number_of_requests_per_client;
        total_number_of_requests
            * (self.max_response_buffer_size
                + self.max_borrowed_responses_per_pending_response
                + max_loaned_responses_per_request)
    }

    /// Returns the request type details of the [`crate::service::Service`].
    pub fn request_message_type_details(&self) -> &MessageTypeDetails {
        &self.request_message_type_details
    }

    /// Returns the response type details of the [`crate::service::Service`].
    pub fn response_message_type_details(&self) -> &MessageTypeDetails {
        &self.response_message_type_details
    }

    /// Returns true if the request buffer of the [`crate::service::Service`] safely overflows,
    /// otherwise false. Safe overflow means that the [`crate::port::client::Client`] will
    /// recycle the oldest requests from the [`crate::port::server::Server`] when its buffer
    /// is full.
    pub fn has_safe_overflow_for_requests(&self) -> bool {
        self.enable_safe_overflow_for_requests
    }

    /// Returns true if the response buffer of the [`crate::service::Service`] safely overflows,
    /// otherwise false. Safe overflow means that the [`crate::port::server::Server`] will
    /// recycle the oldest responses from the [`crate::port::client::Client`] when its buffer
    /// is full.
    pub fn has_safe_overflow_for_responses(&self) -> bool {
        self.enable_safe_overflow_for_responses
    }

    /// Returns true if fire and forget [`RequestMut`](crate::request_mut::RequestMut)s can be
    /// sent from the [`Client`](crate::port::client::Client), otherwise false.
    pub fn does_support_fire_and_forget_requests(&self) -> bool {
        self.enable_fire_and_forget_requests
    }

    /// Returns the maximum number of borrowed [`Response`](crate::response::Response)s a
    /// [`Client`](`crate::port::client::Client`) can hold in
    /// parallel per [`PendingResponse`](crate::pending_response::PendingResponse)
    pub fn max_borrowed_responses_per_pending_response(&self) -> usize {
        self.max_borrowed_responses_per_pending_response
    }

    /// Returns the maximum of active requests a [`crate::port::server::Server`] can hold in
    /// parallel per [`crate::port::client::Client`].
    pub fn max_active_requests_per_client(&self) -> usize {
        self.max_active_requests_per_client
    }

    /// Returns the maximum buffer size for responses for an active request.
    pub fn max_response_buffer_size(&self) -> usize {
        self.max_response_buffer_size
    }

    /// Returns the maximum number of [`RequestMut`](crate::request_mut::RequestMut) a
    /// [`Client`](crate::port::client::Client) can loan in parallel.
    pub fn max_loaned_requests(&self) -> usize {
        self.max_loaned_requests
    }

    /// Returns the maximum number of supported [`crate::port::server::Server`] ports for the
    /// [`crate::service::Service`].
    pub fn max_servers(&self) -> usize {
        self.max_servers
    }

    /// Returns the maximum number of supported [`crate::port::client::Client`] ports for the
    /// [`crate::service::Service`].
    pub fn max_clients(&self) -> usize {
        self.max_clients
    }

    /// Returns the maximum number of supported [`crate::node::Node`]s for the
    /// [`crate::service::Service`].
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }
}
