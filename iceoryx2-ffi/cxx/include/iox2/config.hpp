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

#ifndef IOX2_CONFIG_HPP
#define IOX2_CONFIG_HPP

#include "iox/file_name.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
class Config;

namespace config {
class Global {
  public:
    auto prefix() && -> const char*;
    auto set_prefix(const char* value) &&;

  private:
    friend class ::iox2::Config;
    explicit Global(Config* config);

    Config* m_config;
};
} // namespace config

/// Non-owning view of a [`Config`].
class ConfigView {
  public:
    ConfigView(const ConfigView&) = default;
    ConfigView(ConfigView&&) = default;
    auto operator=(const ConfigView&) -> ConfigView& = default;
    auto operator=(ConfigView&&) -> ConfigView& = default;
    ~ConfigView() = default;

    /// Creates a copy of the corresponding [`Config`] and returns it.
    auto to_owned() const -> Config;

  private:
    friend class Config;
    template <ServiceType>
    friend class Node;

    template <ServiceType>
    friend class Service;

    explicit ConfigView(iox2_config_ptr ptr);
    iox2_config_ptr m_ptr;
};

class Config {
  public:
    Config();
    Config(const Config& rhs);
    Config(Config&& rhs) noexcept;
    ~Config();

    auto operator=(const Config& rhs) -> Config&;
    auto operator=(Config&& rhs) noexcept -> Config&;

    auto global() -> config::Global;

    /// Returns a [`ConfigView`] to the current global config.
    static auto global_config() -> ConfigView;

  private:
    friend class ConfigView;
    friend class config::Global;
    explicit Config(iox2_config_h handle);
    void drop();

    iox2_config_h m_handle = nullptr;
};
} // namespace iox2

#endif
