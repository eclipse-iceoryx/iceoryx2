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

#ifndef IOX2_PORTFACTORY_SUBSCRIBER_HPP_
#define IOX2_PORTFACTORY_SUBSCRIBER_HPP_

#include <cstdint>

#include "iox/assertions_addendum.hpp"
#include "iox/builder.hpp"
#include "iox/expected.hpp"
#include "service_type.hpp"
#include "subscriber.hpp"

namespace iox2 {

template <ServiceType S, typename Payload, typename UserHeader>
class PortFactorySubscriber {
    IOX_BUILDER_PARAMETER(int64_t, history_size, -1)

   public:
    iox::expected<Subscriber<S, Payload, UserHeader>, SubscriberCreateError>
    create() && {
        IOX_TODO();
    }
};
}  // namespace iox2

#endif
