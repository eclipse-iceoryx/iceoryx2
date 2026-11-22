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

#ifndef IOX2_SERVICE_BUILDER_HPP
#define IOX2_SERVICE_BUILDER_HPP

#include "iox2/service_builder_blackboard.hpp"
#include "iox2/service_builder_event.hpp"
#include "iox2/service_builder_publish_subscribe.hpp"
#include "iox2/service_builder_request_response.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// Builder to create or open [`Service`]s
template <ServiceType S>
class ServiceBuilder {
  public:
    ServiceBuilder(ServiceBuilder&& rhs) noexcept;
    auto operator=(ServiceBuilder&& rhs) noexcept -> ServiceBuilder&;
    ~ServiceBuilder();

    ServiceBuilder(const ServiceBuilder&) = delete;
    auto operator=(const ServiceBuilder&) -> ServiceBuilder& = delete;

    /// Create a new builder to create a
    /// [`MessagingPattern::PublishSubscribe`] [`Service`].
    template <typename Payload>
    auto publish_subscribe() && -> ServiceBuilderPublishSubscribe<Payload, void, S>;

    /// Create a new builder to create a
    /// [`MessagingPattern::Event`] [`Service`].
    auto event() && -> ServiceBuilderEvent<S>;

    /// Create a new builder to create a
    /// [`MessagingPattern::RequestResponse`] [`Service`].
    template <typename RequestPayload, typename ResponsePayload>
    auto request_response() && -> ServiceBuilderRequestResponse<RequestPayload, void, ResponsePayload, void, S>;

    /// Create a new builder to create a
    /// [`MessagingPattern::Blackboard`] [`Service`].
    template <typename KeyType>
    auto blackboard_creator() && -> ServiceBuilderBlackboardCreator<KeyType, S>;

    /// Create a new builder to open a
    /// [`MessagingPattern::Blackboard`] [`Service`].
    template <typename KeyType>
    auto blackboard_opener() && -> ServiceBuilderBlackboardOpener<KeyType, S>;

  private:
    template <ServiceType>
    friend class Node;
    ServiceBuilder(iox2_node_h_ref node_handle, iox2_service_name_ptr service_name_ptr);
    void drop();

    iox2_service_builder_h m_handle = nullptr;
};

template <ServiceType S>
inline ServiceBuilder<S>::ServiceBuilder(iox2_node_h_ref node_handle, iox2_service_name_ptr service_name_ptr)
    : m_handle { iox2_node_service_builder(node_handle, nullptr, service_name_ptr) } {
}

template <ServiceType S>
inline void ServiceBuilder<S>::drop() {
    if (m_handle != nullptr) {
        iox2_service_builder_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S>
inline ServiceBuilder<S>::ServiceBuilder(ServiceBuilder&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
inline auto ServiceBuilder<S>::operator=(ServiceBuilder&& rhs) noexcept -> ServiceBuilder& {
    if (this != &rhs) {
        drop();
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
inline ServiceBuilder<S>::~ServiceBuilder() {
    drop();
}

template <ServiceType S>
inline auto ServiceBuilder<S>::event() && -> ServiceBuilderEvent<S> {
    auto* handle = m_handle;
    m_handle = nullptr;
    return ServiceBuilderEvent<S> { handle };
}

template <ServiceType S>
template <typename Payload>
inline auto ServiceBuilder<S>::publish_subscribe() && -> ServiceBuilderPublishSubscribe<Payload, void, S> {
    auto* handle = m_handle;
    m_handle = nullptr;
    return ServiceBuilderPublishSubscribe<Payload, void, S> { handle };
}

template <ServiceType S>
template <typename RequestPayload, typename ResponsePayload>
inline auto ServiceBuilder<
    S>::request_response() && -> ServiceBuilderRequestResponse<RequestPayload, void, ResponsePayload, void, S> {
    auto* handle = m_handle;
    m_handle = nullptr;
    return ServiceBuilderRequestResponse<RequestPayload, void, ResponsePayload, void, S> { handle };
}

template <ServiceType S>
template <typename KeyType>
inline auto ServiceBuilder<S>::blackboard_creator() && -> ServiceBuilderBlackboardCreator<KeyType, S> {
    auto* handle = m_handle;
    m_handle = nullptr;
    return ServiceBuilderBlackboardCreator<KeyType, S> { handle };
}

template <ServiceType S>
template <typename KeyType>
inline auto ServiceBuilder<S>::blackboard_opener() && -> ServiceBuilderBlackboardOpener<KeyType, S> {
    auto* handle = m_handle;
    m_handle = nullptr;
    return ServiceBuilderBlackboardOpener<KeyType, S> { handle };
}
} // namespace iox2
#endif
