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

#![allow(non_camel_case_types)]

use iceoryx2::service::static_config::request_response::StaticConfig;

use super::iox2_message_type_details_t;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_static_config_request_response_t {
    pub enable_safe_overflow_for_requests: bool,
    pub enable_safe_overflow_for_responses: bool,
    pub enable_fire_and_forget_requests: bool,
    pub max_active_requests_per_client: usize,
    pub max_loaned_requests: usize,
    pub max_response_buffer_size: usize,
    pub max_servers: usize,
    pub max_clients: usize,
    pub max_nodes: usize,
    pub max_borrowed_responses_per_pending_response: usize,
    pub request_message_type_details: iox2_message_type_details_t,
    pub response_message_type_details: iox2_message_type_details_t,
}

impl From<&StaticConfig> for iox2_static_config_request_response_t {
    fn from(c: &StaticConfig) -> Self {
        Self {
            enable_safe_overflow_for_requests: c.has_safe_overflow_for_requests(),
            enable_safe_overflow_for_responses: c.has_safe_overflow_for_responses(),
            enable_fire_and_forget_requests: c.does_support_fire_and_forget_requests(),
            max_active_requests_per_client: c.max_active_requests_per_client(),
            max_loaned_requests: c.max_loaned_requests(),
            max_response_buffer_size: c.max_response_buffer_size(),
            max_servers: c.max_servers(),
            max_clients: c.max_clients(),
            max_nodes: c.max_nodes(),
            max_borrowed_responses_per_pending_response: c
                .max_borrowed_responses_per_pending_response(),
            request_message_type_details: c.request_message_type_details().into(),
            response_message_type_details: c.response_message_type_details().into(),
        }
    }
}
