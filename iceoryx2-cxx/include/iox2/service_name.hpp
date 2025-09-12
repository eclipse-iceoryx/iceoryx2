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

#ifndef IOX2_SERVICE_NAME_HPP
#define IOX2_SERVICE_NAME_HPP

#include "iox/expected.hpp"
#include "iox/string.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/semantic_string.hpp"

namespace iox2 {
class ServiceName;

/// Non-owning view of a [`ServiceName`].
class ServiceNameView {
  public:
    ServiceNameView(ServiceNameView&&) = default;
    ServiceNameView(const ServiceNameView&) = default;
    auto operator=(ServiceNameView&&) -> ServiceNameView& = default;
    auto operator=(const ServiceNameView&) -> ServiceNameView& = default;
    ~ServiceNameView() = default;

    /// Returns a [`iox::string`] containing the [`ServiceName`].
    auto to_string() const -> iox::string<IOX2_NODE_NAME_LENGTH>;

    /// Creates a copy of the corresponding [`ServiceName`] and returns it.
    auto to_owned() const -> ServiceName;

  private:
    friend class ServiceName;
    template <ServiceType>
    friend class Service;
    template <ServiceType>
    friend class Node;
    template <ServiceType>
    friend class PortFactoryEvent;
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit ServiceNameView(iox2_service_name_ptr ptr);
    iox2_service_name_ptr m_ptr = nullptr;
};

/// The name of a [`Service`].
class ServiceName {
  public:
    ServiceName(ServiceName&&) noexcept;
    auto operator=(ServiceName&&) noexcept -> ServiceName&;
    ServiceName(const ServiceName&);
    auto operator=(const ServiceName&) -> ServiceName&;
    ~ServiceName();

    /// Creates a [`ServiceNameView`]
    auto as_view() const -> ServiceNameView;

    /// Creates a new [`ServiceName`]. The name is not allowed to be empty.
    static auto create(const char* value) -> iox::expected<ServiceName, SemanticStringError>;

    /// Returns a [`iox::string`] containing the [`ServiceName`].
    auto to_string() const -> iox::string<IOX2_SERVICE_NAME_LENGTH>;

  private:
    friend class ServiceNameView;
    explicit ServiceName(iox2_service_name_h handle);
    static auto create_impl(const char* value, size_t value_len) -> iox::expected<ServiceName, SemanticStringError>;
    void drop() noexcept;

    iox2_service_name_h m_handle = nullptr;
};
} // namespace iox2

#endif
