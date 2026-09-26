// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#ifndef IOX2_LISTENER_KEY_HPP
#define IOX2_LISTENER_KEY_HPP

#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
class MonofierView;

/// An owned key for a listener connection. It can outlive the callback which
/// produced it. Notification with a stale key returns InvalidListenerKey.
class ListenerKey {
  public:
    ListenerKey(const ListenerKey& rhs);
    ListenerKey(ListenerKey&& rhs) noexcept;
    auto operator=(const ListenerKey& rhs) -> ListenerKey&;
    auto operator=(ListenerKey&& rhs) noexcept -> ListenerKey&;
    ~ListenerKey() noexcept;

  private:
    friend class MonofierView;
    template <ServiceType>
    friend class Notifier;

    explicit ListenerKey(iox2_listener_key_h handle);
    void drop() noexcept;
    iox2_listener_key_h m_handle = nullptr;
};
} // namespace iox2

#endif
