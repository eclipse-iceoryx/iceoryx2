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

#ifndef IOX2_PORTFACTORY_PUBLISH_SUBSCRIBE_HPP
#define IOX2_PORTFACTORY_PUBLISH_SUBSCRIBE_HPP

#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/dynamic_config_publish_subscribe.hpp"
#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_failure_enums.hpp"
#include "iox2/node_state.hpp"
#include "iox2/port_factory_publisher.hpp"
#include "iox2/port_factory_subscriber.hpp"
#include "iox2/service_id.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/static_config_publish_subscribe.hpp"

namespace iox2 {
/// The factory for [`MessagingPattern::PublishSubscribe`].
/// It can acquire dynamic and static service informations and create
/// [`Publisher`] or [`Subscriber`] ports.
template <ServiceType S, typename Payload, typename UserHeader>
class PortFactoryPublishSubscribe {
  public:
    PortFactoryPublishSubscribe(PortFactoryPublishSubscribe&& rhs) noexcept;
    auto operator=(PortFactoryPublishSubscribe&& rhs) noexcept -> PortFactoryPublishSubscribe&;
    ~PortFactoryPublishSubscribe();

    PortFactoryPublishSubscribe(const PortFactoryPublishSubscribe&) = delete;
    auto operator=(const PortFactoryPublishSubscribe&) -> PortFactoryPublishSubscribe& = delete;

    /// Returns the [`ServiceName`] of the service
    auto name() const -> ServiceNameView;

    /// Returns the [`ServiceId`] of the [`Service`]
    auto service_id() const -> ServiceId;

    /// Returns the attributes defined in the [`Service`]
    auto attributes() const -> AttributeSetView;

    /// Returns the StaticConfig of the [`Service`].
    /// Contains all settings that never change during the lifetime of the service.
    auto static_config() const -> StaticConfigPublishSubscribe;

    /// Returns the DynamicConfig of the [`Service`].
    /// Contains all dynamic settings, like the current participants etc..
    auto dynamic_config() const -> DynamicConfigPublishSubscribe;

    /// Iterates over all [`Node`]s of the [`Service`]
    /// and calls for every [`Node`] the provided callback. If an error occurs
    /// while acquiring the [`Node`]s corresponding [`NodeState`] the error is
    /// forwarded to the callback as input argument.
    auto nodes(const iox::function<CallbackProgression(NodeState<S>)>& callback) const
        -> iox::expected<void, NodeListFailure>;

    /// Returns a [`PortFactorySubscriber`] to create a new [`Subscriber`] port.
    auto subscriber_builder() const -> PortFactorySubscriber<S, Payload, UserHeader>;

    /// Returns a [`PortFactoryPublisher`] to create a new [`Publisher`] port.
    auto publisher_builder() const -> PortFactoryPublisher<S, Payload, UserHeader>;

  private:
    template <typename, typename, ServiceType>
    friend class ServiceBuilderPublishSubscribe;

    explicit PortFactoryPublishSubscribe(iox2_port_factory_pub_sub_h handle);
    void drop();

    iox2_port_factory_pub_sub_h m_handle = nullptr;
};

template <ServiceType S, typename Payload, typename UserHeader>
inline PortFactoryPublishSubscribe<S, Payload, UserHeader>::PortFactoryPublishSubscribe(
    iox2_port_factory_pub_sub_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline void PortFactoryPublishSubscribe<S, Payload, UserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_port_factory_pub_sub_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename Payload, typename UserHeader>
inline PortFactoryPublishSubscribe<S, Payload, UserHeader>::PortFactoryPublishSubscribe(
    PortFactoryPublishSubscribe&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::operator=(PortFactoryPublishSubscribe&& rhs) noexcept
    -> PortFactoryPublishSubscribe& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename Payload, typename UserHeader>
inline PortFactoryPublishSubscribe<S, Payload, UserHeader>::~PortFactoryPublishSubscribe() {
    drop();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::name() const -> ServiceNameView {
    const auto* service_name_ptr = iox2_port_factory_pub_sub_service_name(&m_handle);
    return ServiceNameView(service_name_ptr);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::service_id() const -> ServiceId {
    iox::UninitializedArray<char, IOX2_SERVICE_ID_LENGTH> buffer;
    iox2_port_factory_pub_sub_service_id(&m_handle, &buffer[0], IOX2_SERVICE_ID_LENGTH);

    return ServiceId(iox::string<IOX2_SERVICE_ID_LENGTH>(iox::TruncateToCapacity, &buffer[0]));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::attributes() const -> AttributeSetView {
    return AttributeSetView(iox2_port_factory_pub_sub_attributes(&m_handle));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::static_config() const -> StaticConfigPublishSubscribe {
    iox2_static_config_publish_subscribe_t static_config {};
    iox2_port_factory_pub_sub_static_config(&m_handle, &static_config);

    return StaticConfigPublishSubscribe(static_config);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::dynamic_config() const
    -> DynamicConfigPublishSubscribe {
    return DynamicConfigPublishSubscribe(m_handle);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::nodes(
    const iox::function<CallbackProgression(NodeState<S>)>& callback) const -> iox::expected<void, NodeListFailure> {
    auto ctx = internal::ctx(callback);

    const auto ret_val =
        iox2_port_factory_pub_sub_nodes(&m_handle, internal::list_callback<S>, static_cast<void*>(&ctx));

    if (ret_val == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<NodeListFailure>(ret_val));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::subscriber_builder() const
    -> PortFactorySubscriber<S, Payload, UserHeader> {
    return PortFactorySubscriber<S, Payload, UserHeader>(
        iox2_port_factory_pub_sub_subscriber_builder(&m_handle, nullptr));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublishSubscribe<S, Payload, UserHeader>::publisher_builder() const
    -> PortFactoryPublisher<S, Payload, UserHeader> {
    return PortFactoryPublisher<S, Payload, UserHeader>(
        iox2_port_factory_pub_sub_publisher_builder(&m_handle, nullptr));
}


} // namespace iox2

#endif
