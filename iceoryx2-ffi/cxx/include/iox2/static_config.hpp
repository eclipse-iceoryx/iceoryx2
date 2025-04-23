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

#ifndef IOX2_STATIC_CONFIG_HPP
#define IOX2_STATIC_CONFIG_HPP

#include "iox2/attribute_set.hpp"
#include "iox2/messaging_pattern.hpp"

namespace iox2 {
/// Defines a common set of static service configuration details every service shares.
class StaticConfig {
  public:
    StaticConfig(const StaticConfig&) = delete;
    StaticConfig(StaticConfig&& rhs) noexcept;
    ~StaticConfig();

    auto operator=(const StaticConfig&) -> StaticConfig& = delete;
    auto operator=(StaticConfig&& rhs) noexcept -> StaticConfig&;

    /// Returns the attributes of the [`Service`]
    auto attributes() const -> AttributeSetView;

    /// Returns the id of the [`Service`]
    auto id() const -> const char*;

    /// Returns the [`ServiceName`] of the [`Service`]
    auto name() const -> const char*;

    /// Returns the [`MessagingPattern`] of the [`Service`]
    auto messaging_pattern() const -> MessagingPattern;

  private:
    template <ServiceType>
    friend class Service;
    template <ServiceType>
    friend auto list_callback(const iox2_static_config_t*, void*) -> iox2_callback_progression_e;
    explicit StaticConfig(iox2_static_config_t value);
    void drop();

    iox2_static_config_t m_value;
};
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::StaticConfig& value) -> std::ostream&;


#endif
