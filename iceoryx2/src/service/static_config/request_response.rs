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

use serde::{Deserialize, Serialize};

use crate::config;

use super::message_type_details::MessageTypeDetails;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StaticConfig {
    pub(crate) enable_safe_overflow_for_requests: bool,
    pub(crate) enable_safe_overflow_for_responses: bool,
    pub(crate) max_active_requests: usize,
    pub(crate) max_borrowed_responses: usize,
    pub(crate) response_buffer_size: usize,
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
            max_active_requests: config.defaults.request_response.max_active_requests,
            max_borrowed_responses: config.defaults.request_response.max_borrowed_responses,
            response_buffer_size: config.defaults.request_response.response_buffer_size,
            max_servers: config.defaults.request_response.max_servers,
            max_clients: config.defaults.request_response.max_clients,
            max_nodes: config.defaults.request_response.max_nodes,
            request_message_type_details: MessageTypeDetails::default(),
            response_message_type_details: MessageTypeDetails::default(),
        }
    }

    pub fn enable_safe_overflow_for_requests(&self) -> bool {
        self.enable_safe_overflow_for_requests
    }

    pub fn enable_safe_overflow_for_responses(&self) -> bool {
        self.enable_safe_overflow_for_responses
    }

    pub fn max_active_requests(&self) -> usize {
        self.max_active_requests
    }

    pub fn max_borrowed_responses(&self) -> usize {
        self.max_borrowed_responses
    }

    pub fn response_buffer_size(&self) -> usize {
        self.response_buffer_size
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
