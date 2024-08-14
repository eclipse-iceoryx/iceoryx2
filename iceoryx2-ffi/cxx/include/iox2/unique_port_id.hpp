// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#ifndef IOX2_UNIQUE_PORT_ID_HPP
#define IOX2_UNIQUE_PORT_ID_HPP

#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
class UniquePublisherId {
  public:
    UniquePublisherId(const UniquePublisherId&) = delete;
    UniquePublisherId(UniquePublisherId&& rhs) = delete;
    auto operator=(const UniquePublisherId& rhs) noexcept -> UniquePublisherId&;
    auto operator=(UniquePublisherId&& rhs) noexcept -> UniquePublisherId&;
    ~UniquePublisherId();

    auto operator==(const UniquePublisherId& rhs) -> bool;
    auto operator<(const UniquePublisherId& rhs) -> bool;

  private:
    friend class HeaderPublishSubscribe;
    explicit UniquePublisherId(iox2_unique_publisher_id_h handle);
    void drop();

    iox2_unique_publisher_id_h m_handle;
};

class UniqueSubscriberId {
  public:
    UniqueSubscriberId(const UniqueSubscriberId&) = delete;
    UniqueSubscriberId(UniqueSubscriberId&& rhs) = delete;
    auto operator=(const UniqueSubscriberId& rhs) noexcept -> UniqueSubscriberId&;
    auto operator=(UniqueSubscriberId&& rhs) noexcept -> UniqueSubscriberId&;
    ~UniqueSubscriberId();

    auto operator==(const UniqueSubscriberId& rhs) -> bool;
    auto operator<(const UniqueSubscriberId& rhs) -> bool;

  private:
    explicit UniqueSubscriberId(iox2_unique_subscriber_id_h handle);
    void drop();

    iox2_unique_subscriber_id_h m_handle;
};

class UniqueNotifierId {
  public:
    UniqueNotifierId(const UniqueNotifierId&) = delete;
    UniqueNotifierId(UniqueNotifierId&& rhs) = delete;
    auto operator=(const UniqueNotifierId& rhs) noexcept -> UniqueNotifierId&;
    auto operator=(UniqueNotifierId&& rhs) noexcept -> UniqueNotifierId&;
    ~UniqueNotifierId();

    auto operator==(const UniqueNotifierId& rhs) -> bool;
    auto operator<(const UniqueNotifierId& rhs) -> bool;

  private:
    explicit UniqueNotifierId(iox2_unique_notifier_id_h handle);
    void drop();

    iox2_unique_notifier_id_h m_handle;
};

class UniqueListenerId {
  public:
    UniqueListenerId(const UniqueListenerId&) = delete;
    UniqueListenerId(UniqueListenerId&& rhs) = delete;
    auto operator=(const UniqueListenerId& rhs) noexcept -> UniqueListenerId&;
    auto operator=(UniqueListenerId&& rhs) noexcept -> UniqueListenerId&;
    ~UniqueListenerId();

    auto operator==(const UniqueListenerId& rhs) -> bool;
    auto operator<(const UniqueListenerId& rhs) -> bool;

  private:
    explicit UniqueListenerId(iox2_unique_listener_id_h handle);
    void drop();

    iox2_unique_listener_id_h m_handle;
};
} // namespace iox2

#endif
