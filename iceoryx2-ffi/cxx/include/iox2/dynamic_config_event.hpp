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

#ifndef IOX2_DYNAMIC_CONFIG_EVENT_HPP
#define IOX2_DYNAMIC_CONFIG_EVENT_HPP

#include "iox/assertions_addendum.hpp"

#include <cstdint>

namespace iox2 {
class DynamicConfigEvent {
  public:
    auto number_of_listeners() const -> uint64_t {
        IOX_TODO();
    }
    auto number_of_notifiers() const -> uint64_t {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
