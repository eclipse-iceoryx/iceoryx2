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

#ifndef IOX2_SUBSCRIBER_DETAILS_HPP
#define IOX2_SUBSCRIBER_DETAILS_HPP

#include "iox2/internal//callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_id.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Contains the communication settings of the connected [`Subscriber`].
class SubscriberDetailsView {
  public:
    SubscriberDetailsView(const SubscriberDetailsView&) = delete;
    SubscriberDetailsView(SubscriberDetailsView&& rhs) noexcept;
    ~SubscriberDetailsView() noexcept = default;

    auto operator=(const SubscriberDetailsView&) -> SubscriberDetailsView& = delete;
    auto operator=(SubscriberDetailsView&& rhs) noexcept -> SubscriberDetailsView&;

    /// The [`UniqueSubscriberId`] of the [`Subscriber`].
    auto subscriber_id() const -> UniqueSubscriberId;

    /// The [`NodeId`] of the [`Node`] under which the [`Subscriber`] was created.
    auto node_id() const -> NodeId;

    /// The receive buffer size for incoming samples.
    auto buffer_size() const -> uint64_t;

  private:
    template <typename T, typename>
    friend auto internal::list_ports_callback(void* context, T port_details_view) -> iox2_callback_progression_e;

    explicit SubscriberDetailsView(iox2_subscriber_details_ptr handle);
    iox2_subscriber_details_ptr m_handle = nullptr;
};
} // namespace iox2

#endif
