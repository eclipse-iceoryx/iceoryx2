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

#include "iox2/dynamic_config_request_response.hpp"
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
auto DynamicConfigRequestResponse::number_of_clients() const -> uint64_t {
    return iox2_port_factory_request_response_dynamic_config_number_of_clients(&m_handle);
}

auto DynamicConfigRequestResponse::number_of_servers() const -> uint64_t {
    return iox2_port_factory_request_response_dynamic_config_number_of_servers(&m_handle);
}

DynamicConfigRequestResponse::DynamicConfigRequestResponse(iox2_port_factory_request_response_h handle)
    : m_handle(handle) {
}

void DynamicConfigRequestResponse::list_servers(
    const iox::function<CallbackProgression(ServerDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_request_response_dynamic_config_list_servers(
        &m_handle, internal::list_ports_callback<iox2_server_details_ptr, ServerDetailsView>, static_cast<void*>(&ctx));
}

void DynamicConfigRequestResponse::list_clients(
    const iox::function<CallbackProgression(ClientDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_request_response_dynamic_config_list_clients(
        &m_handle, internal::list_ports_callback<iox2_client_details_ptr, ClientDetailsView>, static_cast<void*>(&ctx));
}
} // namespace iox2
