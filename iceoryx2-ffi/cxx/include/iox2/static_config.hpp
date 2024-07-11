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

#include <string>

#include "attribute_set.hpp"
#include "iox/assertions_addendum.hpp"
#include "service_name.hpp"

namespace iox2 {
class StaticConfig {
  public:
    auto attributes() const -> const AttributeSet& {
        IOX_TODO();
    }
    auto uuid() const -> std::string {
        IOX_TODO();
    }
    auto name() const -> ServiceName {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
