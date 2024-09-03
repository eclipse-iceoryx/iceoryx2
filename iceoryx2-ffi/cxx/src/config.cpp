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

#include "iox2/config.hpp"

namespace iox2 {
ConfigView::ConfigView(iox2_config_ptr ptr)
    : m_ptr { ptr } {
}

auto ConfigView::to_owned() const -> Config {
    iox2_config_h handle = nullptr;
    iox2_config_from_ptr(m_ptr, nullptr, &handle);

    return Config(handle);
}

Config::Config() {
    iox2_config_default(nullptr, &m_handle);
}

void Config::drop() {
    if (m_handle != nullptr) {
        iox2_config_drop(m_handle);
        m_handle = nullptr;
    }
}

Config::Config(const Config& rhs) {
    auto* ref_handle = iox2_cast_config_ref_h(rhs.m_handle);
    iox2_config_clone(ref_handle, nullptr, &m_handle);
}

Config::Config(Config&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

Config::~Config() {
    drop();
}

auto Config::operator=(const Config& rhs) -> Config& {
    if (this != &rhs) {
        drop();
        auto* ref_handle = iox2_cast_config_ref_h(rhs.m_handle);
        iox2_config_clone(ref_handle, nullptr, &m_handle);
    }
    return *this;
}


auto Config::operator=(Config&& rhs) noexcept -> Config& {
    if (this != &rhs) {
        drop();
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

Config::Config(iox2_config_h handle)
    : m_handle { handle } {
}

auto Config::global() -> config::Global {
    return config::Global(this);
}

auto Config::global_config() -> ConfigView {
    return ConfigView { iox2_config_global_config() };
}

namespace config {
Global::Global(Config* config)
    : m_config { config } {
}

auto Global::prefix() && -> const char* {
    auto* ref_handle = iox2_cast_config_ref_h(m_config->m_handle);
    return iox2_config_global_prefix(ref_handle);
}

void Global::set_prefix(const iox::FileName& value) && {
    auto* ref_handle = iox2_cast_config_ref_h(m_config->m_handle);
    iox2_config_global_set_prefix(ref_handle, value.as_string().c_str());
}
} // namespace config
} // namespace iox2
