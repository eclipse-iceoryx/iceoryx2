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

#include "iox/optional.hpp"
#include "iox/vector.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {

constexpr uint64_t UNIQUE_PORT_ID_LENGTH = 128;
using RawIdType = iox::vector<uint8_t, UNIQUE_PORT_ID_LENGTH>;

/// The system-wide unique id of a [`Publisher`].
class UniquePublisherId {
  public:
    UniquePublisherId(const UniquePublisherId&) = delete;
    UniquePublisherId(UniquePublisherId&& rhs) noexcept;
    auto operator=(const UniquePublisherId& rhs) -> UniquePublisherId& = delete;
    auto operator=(UniquePublisherId&& rhs) noexcept -> UniquePublisherId&;
    ~UniquePublisherId();

    auto bytes() const -> const iox::optional<RawIdType>&;

  private:
    template <ServiceType, typename, typename>
    friend class Publisher;
    friend class HeaderPublishSubscribe;
    friend auto operator==(const UniquePublisherId&, const UniquePublisherId&) -> bool;
    friend auto operator<(const UniquePublisherId&, const UniquePublisherId&) -> bool;

    explicit UniquePublisherId(iox2_unique_publisher_id_h handle);
    void drop();

    iox2_unique_publisher_id_h m_handle = nullptr;
    mutable iox::optional<RawIdType> m_raw_id;
};

/// The system-wide unique id of a [`Subscriber`].
class UniqueSubscriberId {
  public:
    UniqueSubscriberId(const UniqueSubscriberId&) = delete;
    UniqueSubscriberId(UniqueSubscriberId&& rhs) noexcept;
    auto operator=(const UniqueSubscriberId& rhs) -> UniqueSubscriberId& = delete;
    auto operator=(UniqueSubscriberId&& rhs) noexcept -> UniqueSubscriberId&;
    ~UniqueSubscriberId();

    auto bytes() const -> const iox::optional<RawIdType>&;

  private:
    template <ServiceType, typename, typename>
    friend class Subscriber;
    friend auto operator==(const UniqueSubscriberId&, const UniqueSubscriberId&) -> bool;
    friend auto operator<(const UniqueSubscriberId&, const UniqueSubscriberId&) -> bool;

    explicit UniqueSubscriberId(iox2_unique_subscriber_id_h handle);
    void drop();

    iox2_unique_subscriber_id_h m_handle = nullptr;
    mutable iox::optional<RawIdType> m_raw_id;
};

/// The system-wide unique id of a [`Notifier`].
class UniqueNotifierId {
  public:
    UniqueNotifierId(const UniqueNotifierId&) = delete;
    UniqueNotifierId(UniqueNotifierId&& rhs) noexcept;
    auto operator=(const UniqueNotifierId& rhs) -> UniqueNotifierId& = delete;
    auto operator=(UniqueNotifierId&& rhs) noexcept -> UniqueNotifierId&;
    ~UniqueNotifierId();

    auto bytes() const -> const iox::optional<RawIdType>&;

  private:
    template <ServiceType>
    friend class Notifier;
    friend auto operator==(const UniqueNotifierId&, const UniqueNotifierId&) -> bool;
    friend auto operator<(const UniqueNotifierId&, const UniqueNotifierId&) -> bool;

    explicit UniqueNotifierId(iox2_unique_notifier_id_h handle);
    void drop();

    iox2_unique_notifier_id_h m_handle = nullptr;
    mutable iox::optional<RawIdType> m_raw_id;
};

/// The system-wide unique id of a [`Listener`].
class UniqueListenerId {
  public:
    UniqueListenerId(const UniqueListenerId&) = delete;
    UniqueListenerId(UniqueListenerId&& rhs) noexcept;
    auto operator=(const UniqueListenerId& rhs) -> UniqueListenerId& = delete;
    auto operator=(UniqueListenerId&& rhs) noexcept -> UniqueListenerId&;
    ~UniqueListenerId();

    auto bytes() const -> const iox::optional<RawIdType>&;

  private:
    template <ServiceType>
    friend class Listener;
    friend auto operator==(const UniqueListenerId&, const UniqueListenerId&) -> bool;
    friend auto operator<(const UniqueListenerId&, const UniqueListenerId&) -> bool;

    explicit UniqueListenerId(iox2_unique_listener_id_h handle);
    void drop();

    iox2_unique_listener_id_h m_handle = nullptr;
    mutable iox::optional<RawIdType> m_raw_id;
};

auto operator==(const UniquePublisherId& lhs, const UniquePublisherId& rhs) -> bool;
auto operator<(const UniquePublisherId& lhs, const UniquePublisherId& rhs) -> bool;
auto operator==(const UniqueSubscriberId& lhs, const UniqueSubscriberId& rhs) -> bool;
auto operator<(const UniqueSubscriberId& lhs, const UniqueSubscriberId& rhs) -> bool;
auto operator==(const UniqueNotifierId& lhs, const UniqueNotifierId& rhs) -> bool;
auto operator<(const UniqueNotifierId& lhs, const UniqueNotifierId& rhs) -> bool;
auto operator==(const UniqueListenerId& lhs, const UniqueListenerId& rhs) -> bool;
auto operator<(const UniqueListenerId& lhs, const UniqueListenerId& rhs) -> bool;

} // namespace iox2

#endif
