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

#ifndef IOX2_PORTFACTORY_SUBSCRIBER_HPP
#define IOX2_PORTFACTORY_SUBSCRIBER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "service_type.hpp"
#include "subscriber.hpp"

#include <cstdint>

namespace iox2 {

template <ServiceType S, typename Payload, typename UserHeader>
class PortFactorySubscriber {
    IOX_BUILDER_OPTIONAL(uint64_t, history_size);

  public:
    auto create() && -> iox::expected<Subscriber<S, Payload, UserHeader>, SubscriberCreateError> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
