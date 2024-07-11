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

#ifndef IOX2_SUBSCRIBER_HPP_
#define IOX2_SUBSCRIBER_HPP_

#include <cstdint>

#include "connection_failure.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/optional.hpp"
#include "sample.hpp"
#include "service_type.hpp"
#include "unique_port_id.hpp"

namespace iox2 {
enum class SubscriberReceiveError {
};

enum class SubscriberCreateError {
    /// The maximum amount of [`Subscriber`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more
    /// [`Subscriber`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedSubscribers,
    /// When the [`Subscriber`] requires a larger buffer size than the
    /// [`Service`](crate::service::Service) offers the creation will fail.
    BufferSizeExceedsMaxSupportedBufferSizeOfService,
};

template <ServiceType S, typename Payload, typename UserHeader>
class Subscriber {
  public:
    UniqueSubscriberId id() const {
        IOX_TODO();
    }
    uint64_t buffer_size() const {
        IOX_TODO();
    }
    iox::expected<iox::optional<Sample<S, Payload, UserHeader>>, SubscriberReceiveError> receive() const {
        IOX_TODO();
    }
    iox::expected<void, ConnectionFailure> update_connections() const {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
