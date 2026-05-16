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

#include "iox2/attribute_set.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/cleanup_state.hpp"
#include "iox2/dynamic_config_blackboard.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/internal/callback_context.hpp"
#include "iox2/legacy/uninitialized_array.hpp"
#include "iox2/node_state.hpp"
#include "iox2/port_factory_reader.hpp"
#include "iox2/port_factory_writer.hpp"
#include "iox2/service_hash.hpp"
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

    /// Returns the [`ServiceHash`] of the [`Service`]
    auto service_hash() const -> ServiceHash;

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
    auto nodes(const iox2::bb::StaticFunction<CallbackProgression(NodeState<S>)>& callback) const
        -> bb::Expected<void, NodeListFailure>;

    /// Returns a [`PortFactoryWriter`] to create a new [`Writer`] port
    auto writer_builder() const -> PortFactoryWriter<S, KeyType>;

    /// Returns a [`PortFactoryReader`] to create a new [`Reader`] port
    auto reader_builder() const -> PortFactoryReader<S, KeyType>;

    /// Iterates over all keys of the [`Service`] and calls for every key the
    /// provided callback.
    void list_keys(const iox2::bb::StaticFunction<CallbackProgression(const KeyType&)>& callback) const;

    /// Removes the stale system resources of all dead [`Node`]s connected to this service.
    ///
    /// If a [`Node`] cannot be cleaned up since the process has insufficient permissions or it
    /// is currently being cleaned up by another process then the [`Node`] is skipped.
    auto try_cleanup_dead_nodes() const -> CleanupState;

    /// Removes the stale system resources of all dead [`Node`]s connected to this service.
    ///
    /// If a [`Node`] cannot be cleaned up since the process has insufficient permissions then the
    /// [`Node`] is skipped. If it is currently being cleaned up by another process then the
    /// cleaner will wait until the timeout as either passed or the cleaned was finished.
    ///
    /// The timeout is applied to every individual dead [`Node`] the function needs to wait on.
    auto blocking_cleanup_dead_nodes(iox2::bb::Duration timeout) const -> CleanupState;

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
        m_handle = rhs.m_handle;
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
inline auto PortFactoryBlackboard<S, KeyType>::service_hash() const -> ServiceHash {
    iox2::legacy::UninitializedArray<char, IOX2_SERVICE_HASH_LENGTH> buffer;
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-avoid-unchecked-container-access) index 0 is guaranteed to be valid
    iox2_port_factory_blackboard_service_hash(&m_handle, &buffer[0], IOX2_SERVICE_HASH_LENGTH);

    return ServiceHash(iox2::bb::StaticString<IOX2_SERVICE_HASH_LENGTH>::from_utf8_null_terminated_unchecked_truncated(
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-avoid-unchecked-container-access) index 0 is guaranteed to be valid
        &buffer[0],
        IOX2_SERVICE_HASH_LENGTH));
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
    const iox2::bb::StaticFunction<CallbackProgression(NodeState<S>)>& callback) const
    -> bb::Expected<void, NodeListFailure> {
    auto ctx = internal::ctx(callback);

    const auto ret_val =
        iox2_port_factory_blackboard_nodes(&m_handle, internal::list_callback<S>, static_cast<void*>(&ctx));

    if (ret_val == IOX2_OK) {
        return {};
    }

    return bb::err(bb::into<NodeListFailure>(ret_val));
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::try_cleanup_dead_nodes() const -> CleanupState {
    iox2_cleanup_state_t cleanup_state {};

    iox2_port_factory_blackboard_try_cleanup_dead_nodes(&m_handle, &cleanup_state);

    CleanupState ret_val {};
    ret_val.cleanups = cleanup_state.cleanups;
    ret_val.failed_cleanups = cleanup_state.failed_cleanups;
    return ret_val;
}

template <ServiceType T, typename KeyType>
inline auto PortFactoryBlackboard<T, KeyType>::blocking_cleanup_dead_nodes(iox2::bb::Duration timeout) const
    -> CleanupState {
    iox2_cleanup_state_t cleanup_state {};

    iox2_port_factory_blackboard_blocking_cleanup_dead_nodes(
        &m_handle, &cleanup_state, timeout.as_secs(), timeout.subsec_nanos());

    CleanupState ret_val {};
    ret_val.cleanups = cleanup_state.cleanups;
    ret_val.failed_cleanups = cleanup_state.failed_cleanups;
    return ret_val;
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::writer_builder() const -> PortFactoryWriter<S, KeyType> {
    return PortFactoryWriter<S, KeyType>(iox2_port_factory_blackboard_writer_builder(&m_handle, nullptr));
}

template <ServiceType S, typename KeyType>
inline auto PortFactoryBlackboard<S, KeyType>::reader_builder() const -> PortFactoryReader<S, KeyType> {
    return PortFactoryReader<S, KeyType>(iox2_port_factory_blackboard_reader_builder(&m_handle, nullptr));
}

template <typename KeyType>
auto list_keys_callback(const void* const key_ptr, void* ctx) -> iox2_callback_progression_e {
    auto callback = static_cast<iox2::bb::StaticFunction<CallbackProgression(const KeyType&)>*>(ctx);
    auto result = (*callback)(*static_cast<const KeyType* const>(key_ptr));
    return iox2::bb::into<iox2_callback_progression_e>(result);
}

template <ServiceType S, typename KeyType>
inline void PortFactoryBlackboard<S, KeyType>::list_keys(
    const iox2::bb::StaticFunction<CallbackProgression(const KeyType&)>& callback) const {
    auto mutable_callback = callback;
    iox2_port_factory_blackboard_list_keys(&m_handle, list_keys_callback<KeyType>, &mutable_callback);
}
} // namespace iox2

#endif
