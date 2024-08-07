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

#ifndef IOX2_HEADER_PUBLISH_SUBSCRIBE_HPP
#define IOX2_HEADER_PUBLISH_SUBSCRIBE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/layout.hpp"
#include "unique_port_id.hpp"

namespace iox2 {
class HeaderPublishSubscribe {
  public:
    auto publisher_id() const -> UniquePublisherId {
        IOX_TODO();
    }
    auto payload_type_layout() const -> iox::Layout {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
