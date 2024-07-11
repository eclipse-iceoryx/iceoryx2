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

#ifndef IOX2_STATIC_CONFIG_EVENT_HPP
#define IOX2_STATIC_CONFIG_EVENT_HPP

#include "iox/assertions_addendum.hpp"

#include <cstdint>

namespace iox2 {
class StaticConfigEvent {
  public:
    auto max_nodes() const -> uint64_t {
        IOX_TODO();
    }
    auto max_notifiers() const -> uint64_t {
        IOX_TODO();
    }
    auto max_listeners() const -> uint64_t {
        IOX_TODO();
    }
    auto event_id_max_value() const -> uint64_t {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
