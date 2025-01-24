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
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! println!("type details:                     {:?}", pubsub.static_config().message_type_details());
//! println!("max active requests:              {:?}", pubsub.static_config().max_active_requests());
//! println!("max active responses:             {:?}", pubsub.static_config().max_active_responses());
//! println!("max borrowed response samples:    {:?}", pubsub.static_config().max_borrowed_response_samples());
//! println!("max borrowed request samples:     {:?}", pubsub.static_config().max_borrowed_request_samples());
//! println!("max response buffer size:         {:?}", pubsub.static_config().max_response_buffer_size());
//! println!("max request buffer size:          {:?}", pubsub.static_config().max_request_buffer_size());
//! println!("max servers:                      {:?}", pubsub.static_config().max_clients());
//! println!("max clients:                      {:?}", pubsub.static_config().max_servers());
//! println!("max nodes:                        {:?}", pubsub.static_config().max_nodes());
//! println!("request safe overflow:            {:?}", pubsub.static_config().has_safe_overflow_for_requests());
//! println!("response safe overflow:           {:?}", pubsub.static_config().has_safe_overflow_for_responses());
//!
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use crate::config;

use super::message_type_details::MessageTypeDetails;

/// The static configuration of an
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`](crate::service::Service).
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StaticConfig {
    pub(crate) enable_safe_overflow_for_requests: bool,
    pub(crate) enable_safe_overflow_for_responses: bool,
    pub(crate) max_active_responses: usize,
    pub(crate) max_active_requests: usize,
    pub(crate) max_borrowed_response_samples: usize,
    pub(crate) max_borrowed_request_samples: usize,
    pub(crate) max_response_buffer_size: usize,
    pub(crate) max_request_buffer_size: usize,
    pub(crate) max_servers: usize,
    pub(crate) max_clients: usize,
    pub(crate) max_nodes: usize,
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
            max_active_responses: config.defaults.request_response.max_active_responses,
            max_active_requests: config.defaults.request_response.max_active_requests,
            max_borrowed_response_samples: config
                .defaults
                .request_response
                .max_borrowed_response_samples,
            max_borrowed_request_samples: config
                .defaults
                .request_response
                .max_borrowed_request_samples,
            max_response_buffer_size: config.defaults.request_response.max_response_buffer_size,
            max_request_buffer_size: config.defaults.request_response.max_request_buffer_size,
            max_servers: config.defaults.request_response.max_servers,
            max_clients: config.defaults.request_response.max_clients,
            max_nodes: config.defaults.request_response.max_nodes,
            request_message_type_details: MessageTypeDetails::default(),
            response_message_type_details: MessageTypeDetails::default(),
        }
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

    /// Returns the maximum of active responses a [`crate::port::server::Server`] can hold in
    /// parallel.
    pub fn max_active_responses(&self) -> usize {
        self.max_active_responses
    }

    /// Returns the maximum of active requests a [`crate::port::client::Client`] can hold in
    /// parallel.
    pub fn max_active_requests(&self) -> usize {
        self.max_active_requests
    }

    pub fn max_borrowed_response_samples(&self) -> usize {
        self.max_borrowed_response_samples
    }

    pub fn max_borrowed_request_samples(&self) -> usize {
        self.max_borrowed_request_samples
    }

    pub fn max_response_buffer_size(&self) -> usize {
        self.max_response_buffer_size
    }

    pub fn max_request_buffer_size(&self) -> usize {
        self.max_request_buffer_size
    }

    pub fn max_servers(&self) -> usize {
        self.max_servers
    }

    pub fn max_clients(&self) -> usize {
        self.max_clients
    }

    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }
}
