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

#ifndef IOX2_SERVER_DETAILS_HPP
#define IOX2_SERVER_DETAILS_HPP

#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_id.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Contains the communication settings of the connected [`Server`].
class ServerDetailsView {
  public:
    ServerDetailsView(const ServerDetailsView&) = delete;
    ServerDetailsView(ServerDetailsView&& rhs) noexcept;
    ~ServerDetailsView() noexcept = default;

    auto operator=(const ServerDetailsView&) -> ServerDetailsView& = delete;
    auto operator=(ServerDetailsView&& rhs) noexcept -> ServerDetailsView&;

    /// The [`UniqueServerId`] of the [`Server`]
    auto server_id() const -> UniqueServerId;

    /// The [`NodeId`] of the [`Node`] under which the [`Server`] was created.
    auto node_id() const -> NodeId;

    /// The receive buffer size for incoming requests.
    auto request_buffer_size() const -> uint64_t;

    /// The total number of responses available in the [`Server`].
    auto number_of_responses() const -> uint64_t;

    /// The current maximum length of a slice.
    auto max_slice_len() const -> uint64_t;

  private:
    template <typename T, typename>
    friend auto internal::list_ports_callback(void* context, T port_details_view) -> iox2_callback_progression_e;

    explicit ServerDetailsView(iox2_server_details_ptr handle);
    iox2_server_details_ptr m_handle = nullptr;
};
} // namespace iox2
#endif
