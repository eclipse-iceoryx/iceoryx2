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

#ifndef IOX2_NODE_NAME_HPP_
#define IOX2_NODE_NAME_HPP_

#include <string>

#include "internal/iceoryx2.hpp"
#include "iox/expected.hpp"
#include "semantic_string.hpp"

namespace iox2 {
class NodeName {
   public:
    static iox::expected<NodeName, SemanticStringError> create(
        const char* value);

    const std::string& as_string() const;

   private:
    iox2_node_name_storage_t value;
};
}  // namespace iox2

#endif
