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

#ifndef IOX2_NODE_DETAILS_HPP
#define IOX2_NODE_DETAILS_HPP

#include "iox2/config.hpp"
#include "iox2/node_name.hpp"

namespace iox2 {
/// Contains details of a [`Node`].
class NodeDetails {
  public:
    NodeDetails(NodeName name, const Config& config);

    /// Returns a reference of the [`NodeName`].
    auto name() const -> const NodeName&;
    /// Returns a reference to the [`Config`] the [`Node`] uses.
    auto config() const -> const Config&;

  private:
    NodeName m_node_name;
    Config m_config;
};
} // namespace iox2

#endif
