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

#ifndef IOX2_PORTFACTORY_BLACKBOARD_HPP
#define IOX2_PORTFACTORY_BLACKBOARD_HPP

#include "iox/expected.hpp"
#include "iox/uninitialized_array.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/dynamic_config_blackboard.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/internal/callback_context.hpp"
#include "iox2/node_state.hpp"
#include "iox2/port_factory_reader.hpp"
#include "iox2/port_factory_writer.hpp"
#include "iox2/service_id.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/static_config_blackboard.hpp"

namespace iox2 {
/// Represents the port factory of a [`Service`] with [`MessagingPattern::Blackboard`].
template <ServiceType S, typename KeyType>
class PortFactoryBlackboard {
  public:
    PortFactoryBlackboard(PortFactoryBlackboard&& rhs) noexcept;
    auto operator=(PortFactoryBlackboard&& rhs) noexcept -> PortFactoryBlackboard&;
    ~PortFactoryBlackboard();

    PortFactoryBlackboard(const PortFactoryBlackboard&) = delete;
    auto operator=(const PortFactoryBlackboard&) -> PortFactoryBlackboard& = delete;

    /// Returns the [`ServiceName`] of the service
    auto name() const -> ServiceNameView;

    /// Returns the [`ServiceId`] of the [`Service`]
    auto service_id() const -> ServiceId;

    /// Returns the attributes defined in the [`Service`]
    auto attributes() const -> AttributeSetView;

    /// Returns the StaticConfig of the [`Service`].
    /// Contains all settings that never change during the lifetime of the service.
    auto static_config() const -> StaticConfigBlackboard;

    /// Returns the DynamicConfig of the [`Service`].
    /// Contains all dynamic settings, like the current participants etc..
    auto dynamic_config() const -> DynamicConfigBlackboard;

    /// Iterates over all [`Node`]s of the [`Service`]
    /// and calls for every [`Node`] the provided callback. If an error occurs
    /// while acquiring the [`Node`]s corresponding [`NodeState`] the error is
    /// forwarded to the callback as input argument.
    auto nodes(const iox::function<CallbackProgression(NodeState<S>)>& callback) const
        -> iox::expected<void, NodeListFailure>;

    /// Returns a [`PortFactoryWriter`] to create a new [`Writer`] port
    auto writer_builder() const -> PortFactoryWriter<S, KeyType>;

    /// Returns a [`PortFactoryReader`] to create a new [`Reader`] port
    auto reader_builder() const -> PortFactoryReader<S, KeyType>;

  private:
    template <typename, ServiceType>
    friend class ServiceBuilderBlackboardOpener;
    template <typename, ServiceType>
    friend class ServiceBuilderBlackboardCreator;

    explicit PortFactoryBlackboard(iox2_port_factory_blackboard_h handle);
    void drop() noexcept;

    iox2_port_factory_blackboard_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType>
inline PortFactoryBlackboard<S, KeyType>::PortFactoryBlackboard(iox2_port_factory_blackboard_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename KeyType>
inline void PortFactoryBlackboard<S, KeyType>::drop() noexcept {
    if (m_handle != nullptr) {
        iox2_port_factory_blackboard_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename KeyType>
inline PortFactoryBlackboard<S, KeyType>::PortFactoryBlackboard(PortFactoryBlackboard&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::operator=(PortFactoryBlackboard&& rhs) noexcept
    -> PortFactoryBlackboard& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename KeyType>
inline PortFactoryBlackboard<S, KeyType>::~PortFactoryBlackboard() {
    drop();
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::name() const -> ServiceNameView {
    const auto* service_name_ptr = iox2_port_factory_blackboard_service_name(&m_handle);
    return ServiceNameView(service_name_ptr);
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::service_id() const -> ServiceId {
    iox::UninitializedArray<char, IOX2_SERVICE_ID_LENGTH> buffer;
    iox2_port_factory_blackboard_service_id(&m_handle, &buffer[0], IOX2_SERVICE_ID_LENGTH);

    return ServiceId(iox::string<IOX2_SERVICE_ID_LENGTH>(iox::TruncateToCapacity, &buffer[0]));
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::attributes() const -> AttributeSetView {
    return AttributeSetView(iox2_port_factory_blackboard_attributes(&m_handle));
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::static_config() const -> StaticConfigBlackboard {
    iox2_static_config_blackboard_t static_config {};
    iox2_port_factory_blackboard_static_config(&m_handle, &static_config);

    return StaticConfigBlackboard(static_config);
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::dynamic_config() const -> DynamicConfigBlackboard {
    return DynamicConfigBlackboard(m_handle);
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::nodes(
    [[maybe_unused]] const iox::function<CallbackProgression(NodeState<S>)>& callback) const
    -> iox::expected<void, NodeListFailure> {
    auto ctx = internal::ctx(callback);

    const auto ret_val =
        iox2_port_factory_blackboard_nodes(&m_handle, internal::list_callback<S>, static_cast<void*>(&ctx));

    if (ret_val == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<NodeListFailure>(ret_val));
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::writer_builder() const -> PortFactoryWriter<S, KeyType> {
    return PortFactoryWriter<S, KeyType>(iox2_port_factory_blackboard_writer_builder(&m_handle, nullptr));
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::reader_builder() const -> PortFactoryReader<S, KeyType> {
    return PortFactoryReader<S, KeyType>(iox2_port_factory_blackboard_reader_builder(&m_handle, nullptr));
}
} // namespace iox2

#endif
