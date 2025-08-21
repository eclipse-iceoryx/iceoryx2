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

#ifndef IOX2_CLIENT_DETAILS_HPP
#define IOX2_CLIENT_DETAILS_HPP

#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_id.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Contains the communication settings of the connected [`Client`].
class ClientDetailsView {
  public:
    ClientDetailsView(const ClientDetailsView&) = delete;
    ClientDetailsView(ClientDetailsView&& rhs) noexcept;
    ~ClientDetailsView() noexcept = default;

    auto operator=(const ClientDetailsView&) -> ClientDetailsView& = delete;
    auto operator=(ClientDetailsView&& rhs) noexcept -> ClientDetailsView&;

    /// The [`UniqueClientId`] of the [`Client`].
    auto client_id() const -> UniqueClientId;

    /// The [`NodeId`] of the [`Node`] under which the [`Client`] was created.
    auto node_id() const -> NodeId;

    /// The receive buffer size for incoming responses.
    auto response_buffer_size() const -> uint64_t;

    /// The total number of requests available in the [`Client`]s data segment
    auto number_of_requests() const -> uint64_t;

    /// The current maximum length of a slice.
    auto max_slice_len() const -> uint64_t;

  private:
    template <typename T, typename>
    friend auto internal::list_ports_callback(void* context, T port_details_view) -> iox2_callback_progression_e;

    explicit ClientDetailsView(iox2_client_details_ptr handle);
    iox2_client_details_ptr m_handle = nullptr;
};
} // namespace iox2

#endif
