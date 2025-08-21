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

#ifndef IOX2_STATIC_CONFIG_REQUEST_RESPONSE_HPP
#define IOX2_STATIC_CONFIG_REQUEST_RESPONSE_HPP

#include "iox2/message_type_details.hpp"

namespace iox2 {
/// The static configuration of an
/// [`MessagingPattern::RequestResponse`]
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`].
class StaticConfigRequestResponse {
  public:
    /// Returns the request type details of the [`Service`].
    auto request_message_type_details() const -> MessageTypeDetails;

    /// Returns the response type details of the [`Service`].
    auto response_message_type_details() const -> MessageTypeDetails;

    /// Returns true if the request buffer of the [`Service`] safely overflows, otherwise false.
    /// Safe overflow means that the [`Client`] will recycle the oldest requests from the
    /// [`Server`] when its buffer is full.
    auto has_safe_overflow_for_requests() const -> bool;

    /// Returns true if the response buffer of the [`Service`] safely overflows, otherwise false.
    /// Safe overflow means that the [`Server`] will recycle the oldest responses from the
    /// [`Client`] when its buffer is full.
    auto has_safe_overflow_for_responses() const -> bool;

    /// Returns the maximum number of borrowed [`Response`]s a [`Client`] can hold in parallel per
    /// [`PendingResponse`]
    auto max_borrowed_responses_per_pending_responses() const -> uint64_t;

    /// Returns the maximum of active requests a [`Server`] can hold in parallel per [`Client`].
    auto max_active_requests_per_client() const -> uint64_t;

    /// Returns the maximum buffer size for responses for a [`PendingResponse`].
    auto max_response_buffer_size() const -> uint64_t;

    /// Returns the maximum number of [`RequestMut`] a [`Client`] can loan in parallel.
    auto max_loaned_requests() const -> uint64_t;

    /// Returns true if fire and forget [`RequestMut`]s can be sent from the [`Client`], otherwise
    /// false.
    auto does_support_fire_and_forget_requests() const -> bool;

    /// Returns the maximum number of supported [`Server`] ports for the [`Service`].
    auto max_servers() const -> uint64_t;

    /// Returns the maximum number of supported [`Client`] ports for the [`Service`].
    auto max_clients() const -> uint64_t;

    /// Returns the maximum number of supported [`Node`]s for the [`Service`].
    auto max_nodes() const -> uint64_t;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit StaticConfigRequestResponse(iox2_static_config_request_response_t value);

    iox2_static_config_request_response_t m_value;
};
} // namespace iox2
#endif
