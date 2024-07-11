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

#ifndef IOX2_SAMPLE_HPP_
#define IOX2_SAMPLE_HPP_

#include "header_publish_subscribe.hpp"
#include "iox/assertions_addendum.hpp"
#include "service_type.hpp"
#include "unique_port_id.hpp"

namespace iox2 {

template <ServiceType, typename Payload, typename UserHeader>
class Sample {
  public:
    const Payload& payload() const {
        IOX_TODO();
    }
    const UserHeader& user_header() const {
        IOX_TODO();
    }
    const HeaderPublishSubscribe& header() const {
        IOX_TODO();
    }
    UniquePublisherId origin() const {
        IOX_TODO();
    }
};

template <ServiceType S, typename Payload>
class Sample<S, Payload, void> {
  public:
    const Payload& payload() const {
        IOX_TODO();
    }
    const HeaderPublishSubscribe& header() const {
        IOX_TODO();
    }
    UniquePublisherId origin() const {
        IOX_TODO();
    }
};

} // namespace iox2

#endif
