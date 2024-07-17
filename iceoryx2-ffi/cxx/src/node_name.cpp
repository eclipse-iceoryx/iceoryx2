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

#include "iox2/node_name.hpp"

#include "iox/assertions_addendum.hpp"

namespace iox2 {
auto NodeName::create(const char* value) -> iox::expected<NodeName, SemanticStringError> {
    IOX_TODO();
}

auto NodeName::to_string() const -> iox::string<NODE_NAME_LENGHT> {
    IOX_TODO();
}

} // namespace iox2
