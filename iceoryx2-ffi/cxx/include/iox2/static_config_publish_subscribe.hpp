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

#ifndef IOX2_STATIC_CONFIG_PUBLISH_SUBSCRIBE_HPP_
#define IOX2_STATIC_CONFIG_PUBLISH_SUBSCRIBE_HPP_

#include <cstdint>

#include "iox/assertions_addendum.hpp"
#include "message_type_details.hpp"

namespace iox2 {
class StaticConfigPublishSubscribe {
  public:
    uint64_t max_nodes() const {
        IOX_TODO();
    }
    uint64_t max_publishers() const {
        IOX_TODO();
    }
    uint64_t max_subscribers() const {
        IOX_TODO();
    }
    uint64_t history_size() const {
        IOX_TODO();
    }
    uint64_t subscriber_max_buffer_size() const {
        IOX_TODO();
    }
    uint64_t subscriber_max_borrowed_samples() const {
        IOX_TODO();
    }
    bool has_safe_overflow() const {
        IOX_TODO();
    }
    const MessageTypeDetails& message_type_details() const {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
