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

#include "iox2/node_details.hpp"

namespace iox2 {
NodeDetails::NodeDetails(iox::FileName executable, NodeName name, Config config)
    : m_executable { std::move(executable) }
    , m_node_name { std::move(name) }
    , m_config { std::move(config) } {
}

auto NodeDetails::name() const -> const NodeName& {
    return m_node_name;
}

auto NodeDetails::config() const -> const Config& {
    return m_config;
}


} // namespace iox2
