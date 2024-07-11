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

#ifndef IOX2_HEADER_PUBLISH_SUBSCRIBE_HPP_
#define IOX2_HEADER_PUBLISH_SUBSCRIBE_HPP_

#include "iox/assertions_addendum.hpp"
#include "iox/layout.hpp"
#include "service_type.hpp"
#include "unique_port_id.hpp"

namespace iox2 {
class HeaderPublishSubscribe {
  public:
    UniquePublisherId publisher_id() const {
        IOX_TODO();
    }
    iox::Layout payload_type_layout() const {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
