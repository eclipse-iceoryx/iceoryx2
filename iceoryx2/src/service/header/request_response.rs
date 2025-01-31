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

use crate::port::port_identifiers::{UniqueClientId, UniqueServerId};

/// Request header used by
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RequestHeader {
    pub client_port_id: UniqueClientId,
}

/// Response header used by
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ResponseHeader {
    pub server_port_id: UniqueServerId,
}
