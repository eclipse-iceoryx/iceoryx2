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

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::zero_copy_connection::ChannelId;

use crate::port::port_identifiers::{UniqueClientId, UniqueServerId};

/// Request header used by
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
#[derive(Debug, Copy, Clone, ZeroCopySend)]
#[repr(C)]
pub struct RequestHeader {
    pub(crate) client_id: UniqueClientId,
    pub(crate) channel_id: ChannelId,
    pub(crate) request_id: u64,
    pub(crate) number_of_elements: u64,
}

impl RequestHeader {
    /// Returns the [`UniqueClientId`] of the [`Client`](crate::port::client::Client)
    /// which sent the [`RequestMut`](crate::request_mut::RequestMut)
    pub fn client_id(&self) -> UniqueClientId {
        self.client_id
    }

    /// Returns how many elements are stored inside the requests's payload.
    ///
    /// # Details when using
    /// [`CustomPayloadMarker`](crate::service::builder::CustomPayloadMarker)
    ///
    /// In this case the number of elements relates to the element defined in the
    /// [`MessageTypeDetails`](crate::service::static_config::message_type_details::MessageTypeDetails).
    /// When the element has a `payload.size == 40` and the `RequestMut::payload().len() == 120` it
    /// means that it contains 3 elements (3 * 40 == 120).
    pub fn number_of_elements(&self) -> u64 {
        self.number_of_elements
    }
}

/// Response header used by
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
#[derive(Debug, Copy, Clone, ZeroCopySend)]
#[repr(C)]
pub struct ResponseHeader {
    pub(crate) server_id: UniqueServerId,
    pub(crate) request_id: u64,
    pub(crate) number_of_elements: u64,
}

impl ResponseHeader {
    /// Returns the [`UniqueServerId`] of the [`Server`](crate::port::server::Server)
    /// which sent the [`Response`](crate::response::Response)
    pub fn server_id(&self) -> UniqueServerId {
        self.server_id
    }

    /// Returns how many elements are stored inside the [`Response`](crate::response::Response)s
    /// payload.
    ///
    /// # Details when using
    /// [`CustomPayloadMarker`](crate::service::builder::CustomPayloadMarker)
    ///
    /// In this case the number of elements relates to the element defined in the
    /// [`MessageTypeDetails`](crate::service::static_config::message_type_details::MessageTypeDetails).
    /// When the element has a `payload.size == 40` and the `ResponseMut::payload().len() == 120` it
    /// means that it contains 3 elements (3 * 40 == 120).
    pub fn number_of_elements(&self) -> u64 {
        self.number_of_elements
    }
}
