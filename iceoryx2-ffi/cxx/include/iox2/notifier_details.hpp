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

#ifndef IOX2_NOTIFIER_DETAILS_HPP
#define IOX2_NOTIFIER_DETAILS_HPP

#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_id.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Contains the communication settings of the connected [`Notifier`].
class NotifierDetailsView {
  public:
    NotifierDetailsView(const NotifierDetailsView&) = delete;
    NotifierDetailsView(NotifierDetailsView&& rhs) noexcept;
    ~NotifierDetailsView() noexcept = default;

    auto operator=(const NotifierDetailsView&) -> NotifierDetailsView& = delete;
    auto operator=(NotifierDetailsView&& rhs) noexcept -> NotifierDetailsView&;

    /// The [`UniqueNotifierId`] of the [`Notifier`].
    auto notifier_id() const -> UniqueNotifierId;

    /// The [`NodeId`] of the [`Node`] under which the [`Notifier`] was created.
    auto node_id() const -> NodeId;

  private:
    template <typename T, typename>
    friend auto internal::list_ports_callback(void* context, T port_details_view) -> iox2_callback_progression_e;

    explicit NotifierDetailsView(iox2_notifier_details_ptr handle);
    iox2_notifier_details_ptr m_handle = nullptr;
};
} // namespace iox2

#endif
