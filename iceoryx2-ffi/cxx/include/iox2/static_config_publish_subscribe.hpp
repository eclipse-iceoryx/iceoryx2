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

#ifndef IOX2_STATIC_CONFIG_PUBLISH_SUBSCRIBE_HPP
#define IOX2_STATIC_CONFIG_PUBLISH_SUBSCRIBE_HPP

#include "iox/assertions_addendum.hpp"
#include "message_type_details.hpp"

#include <cstdint>

namespace iox2 {
class StaticConfigPublishSubscribe {
  public:
    auto max_nodes() const -> uint64_t {
        IOX_TODO();
    }
    auto max_publishers() const -> uint64_t {
        IOX_TODO();
    }
    auto max_subscribers() const -> uint64_t {
        IOX_TODO();
    }
    auto history_size() const -> uint64_t {
        IOX_TODO();
    }
    auto subscriber_max_buffer_size() const -> uint64_t {
        IOX_TODO();
    }
    auto subscriber_max_borrowed_samples() const -> uint64_t {
        IOX_TODO();
    }
    auto has_safe_overflow() const -> bool {
        IOX_TODO();
    }
    auto message_type_details() const -> const MessageTypeDetails& {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
