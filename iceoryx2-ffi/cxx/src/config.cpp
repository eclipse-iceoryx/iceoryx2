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
    return Config {};
}

auto Config::global_config() -> ConfigView {
    return ConfigView { iox2_config_global_config() };
}
} // namespace iox2
