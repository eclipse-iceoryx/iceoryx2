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

#ifndef IOX2_PORTFACTORY_PUBLISHER_HPP
#define IOX2_PORTFACTORY_PUBLISHER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "publisher.hpp"
#include "service_type.hpp"

#include <cstdint>

namespace iox2 {
enum class UnableToDeliverStrategy : uint8_t {
    Block,
    DiscardSample
};

template <ServiceType S, typename Payload, typename UserHeader>
class PortFactoryPublisher {
    IOX_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);
    IOX_BUILDER_OPTIONAL(uint64_t, max_loaned_samples);
    IOX_BUILDER_OPTIONAL(uint64_t, max_slice_len);

  public:
    auto create() && -> iox::expected<Publisher<S, Payload, UserHeader>, PublisherCreateError> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
