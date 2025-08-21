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

#ifndef IOX2_DYNAMIC_CONFIG_REQUEST_RESPONSE_HPP
#define IOX2_DYNAMIC_CONFIG_REQUEST_RESPONSE_HPP

#include "iox/function.hpp"
#include "iox2/client_details.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/server_details.hpp"
#include "iox2/service_type.hpp"

#include <cstdint>

namespace iox2 {

/// The dynamic configuration of an
/// [`crate::service::messaging_pattern::MessagingPattern::RequestResponse`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
class DynamicConfigRequestResponse {
  public:
    DynamicConfigRequestResponse(const DynamicConfigRequestResponse&) = delete;
    DynamicConfigRequestResponse(DynamicConfigRequestResponse&&) = delete;
    auto operator=(const DynamicConfigRequestResponse&) -> DynamicConfigRequestResponse& = delete;
    auto operator=(DynamicConfigRequestResponse&&) -> DynamicConfigRequestResponse& = delete;
    ~DynamicConfigRequestResponse() = default;

    /// Returns how many [`crate::port::client::Client`] ports are currently connected.
    auto number_of_clients() const -> uint64_t;

    /// Returns how many [`crate::port::server::Server`] ports are currently connected.
    auto number_of_servers() const -> uint64_t;

    /// Iterates over all [`Server`]s and calls the
    /// callback with the corresponding [`ServerDetailsView`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    void list_servers(const iox::function<CallbackProgression(ServerDetailsView)>& callback) const;

    /// Iterates over all [`Client`]s and calls the
    /// callback with the corresponding [`ClientDetailsView`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    void list_clients(const iox::function<CallbackProgression(ClientDetailsView)>& callback) const;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit DynamicConfigRequestResponse(iox2_port_factory_request_response_h handle);

    iox2_port_factory_request_response_h m_handle = nullptr;
};
} // namespace iox2
#endif
