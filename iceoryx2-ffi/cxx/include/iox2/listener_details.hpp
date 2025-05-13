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

#ifndef IOX2_LISTENER_DETAILS_HPP
#define IOX2_LISTENER_DETAILS_HPP

#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_id.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Contains the communication settings of the connected [`Listener`].
class ListenerDetailsView {
  public:
    ListenerDetailsView(const ListenerDetailsView&) = delete;
    ListenerDetailsView(ListenerDetailsView&& rhs) noexcept;
    ~ListenerDetailsView() noexcept = default;

    auto operator=(const ListenerDetailsView&) -> ListenerDetailsView& = delete;
    auto operator=(ListenerDetailsView&& rhs) noexcept -> ListenerDetailsView&;

    /// The [`UniqueListenerId`] of the [`Listener`].
    auto listener_id() const -> UniqueListenerId;

    /// The [`NodeId`] of the [`Node`] under which the [`Listener`] was created.
    auto node_id() const -> NodeId;

  private:
    template <typename T, typename>
    friend auto internal::list_ports_callback(void* context, T port_details_view) -> iox2_callback_progression_e;

    explicit ListenerDetailsView(iox2_listener_details_ptr handle);
    iox2_listener_details_ptr m_handle = nullptr;
};
} // namespace iox2

#endif
