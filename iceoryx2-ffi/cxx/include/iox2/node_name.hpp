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

#ifndef IOX2_NODE_NAME_HPP
#define IOX2_NODE_NAME_HPP

#include "internal/iceoryx2.hpp"
#include "iox/expected.hpp"
#include "iox/string.hpp"
#include "iox2/iceoryx2_settings.hpp"
#include "semantic_string.hpp"

namespace iox2 {
class NodeName {
  public:
    static auto create(const char* value) -> iox::expected<NodeName, SemanticStringError>;

    auto to_string() const -> iox::string<NODE_NAME_LENGTH>;

    auto as_c_str() const -> const char* const;

  private:
    explicit NodeName(iox2_node_name_h handle);

    iox2_node_name_h m_handle;
};
} // namespace iox2

#endif
