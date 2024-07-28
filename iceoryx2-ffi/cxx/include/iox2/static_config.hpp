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

#include "iox/assertions_addendum.hpp"
#include "iox/string.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/messaging_pattern.hpp"
#include "iox2/service_name.hpp"

namespace iox2 {
class StaticConfig {
  public:
    auto attributes() const -> const AttributeSet& {
        IOX_TODO();
    }
    auto id() const -> iox::string<IOX2_SERVICE_ID_LENGTH> {
        IOX_TODO();
    }
    auto name() const -> ServiceName {
        IOX_TODO();
    }
    auto messaging_pattern() const -> MessagingPattern {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
