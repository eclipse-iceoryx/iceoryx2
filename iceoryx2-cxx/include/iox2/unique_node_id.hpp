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

#ifndef IOX2_UNIQUE_NODE_ID_HPP
#define IOX2_UNIQUE_NODE_ID_HPP

#include "iox2/bb/into.hpp"
#include "iox2/enum_translation.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/service_type.hpp"

#include <cstdint>
#include <ctime>
#include <iostream>

namespace iox2 {
namespace internal {
template <ServiceType>
auto list_callback(
    iox2_node_state_e, iox2_unique_node_id_ptr, const char*, iox2_node_name_ptr, iox2_config_ptr, iox2_callback_context)
    -> iox2_callback_progression_e;
}

/// The system-wide unique id of a [`Node`]
class UniqueNodeId {
  public:
    UniqueNodeId(const UniqueNodeId& rhs);
    UniqueNodeId(UniqueNodeId&& rhs) noexcept;
    auto operator=(const UniqueNodeId& rhs) -> UniqueNodeId&;
    auto operator=(UniqueNodeId&& rhs) noexcept -> UniqueNodeId&;
    ~UniqueNodeId();

    /// Returns the high bits of the underlying value of the [`NodeId`].
    auto value_high() const -> uint64_t;

    /// Returns the low bits of the underlying value of the [`NodeId`].
    auto value_low() const -> uint64_t;

    /// Returns the [`ProcessId`] of the process that owns the [`Node`].
    template <ServiceType T>
    auto pid() const -> int32_t;

    /// Returns the time the [`Node`] was created.
    template <ServiceType T>
    auto creation_time() const -> timespec;

  private:
    template <ServiceType>
    friend class Node;
    template <ServiceType>
    friend class DeadNodeView;
    template <ServiceType>
    friend auto internal::list_callback(iox2_node_state_e,
                                        iox2_unique_node_id_ptr,
                                        const char*,
                                        iox2_node_name_ptr,
                                        iox2_config_ptr,
                                        iox2_callback_context) -> iox2_callback_progression_e;
    friend class ClientDetailsView;
    friend class ServerDetailsView;
    friend class NotifierDetailsView;
    friend class ListenerDetailsView;
    friend class PublisherDetailsView;
    friend class ReaderDetailsView;
    friend class SubscriberDetailsView;
    friend class WriterDetailsView;

    explicit UniqueNodeId(iox2_unique_node_id_h handle);
    void drop();

    iox2_unique_node_id_h m_handle = nullptr;
};

template <ServiceType T>
auto UniqueNodeId::pid() const -> int32_t {
    return iox2_unique_node_id_pid(&m_handle, iox2::bb::into<iox2_service_type_e>(T));
}

template <ServiceType T>
auto UniqueNodeId::creation_time() const -> timespec {
    uint64_t seconds = 0;
    uint32_t nanoseconds = 0;
    iox2_unique_node_id_creation_time(&m_handle, iox2::bb::into<iox2_service_type_e>(T), &seconds, &nanoseconds);

    return { static_cast<decltype(timespec::tv_sec)>(seconds), static_cast<decltype(timespec::tv_nsec)>(nanoseconds) };
}

auto operator<<(std::ostream& stream, const UniqueNodeId& node) -> std::ostream&;
auto operator==(const UniqueNodeId& lhs, const UniqueNodeId& rhs) -> bool;
auto operator!=(const UniqueNodeId& lhs, const UniqueNodeId& rhs) -> bool;
} // namespace iox2

#endif
