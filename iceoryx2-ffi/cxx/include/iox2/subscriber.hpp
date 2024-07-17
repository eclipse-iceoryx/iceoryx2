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

#ifndef IOX2_SUBSCRIBER_HPP
#define IOX2_SUBSCRIBER_HPP

#include "connection_failure.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/optional.hpp"
#include "sample.hpp"
#include "service_type.hpp"
#include "unique_port_id.hpp"

#include <cstdint>

namespace iox2 {
enum class SubscriberReceiveError : uint8_t {
};

enum class SubscriberCreateError : uint8_t {
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
    Subscriber() = default;
    Subscriber(Subscriber&&) = default;
    auto operator=(Subscriber&&) -> Subscriber& = default;
    ~Subscriber() = default;

    Subscriber(const Subscriber&) = delete;
    auto operator=(const Subscriber&) -> Subscriber& = delete;

    auto id() const -> UniqueSubscriberId {
        IOX_TODO();
    }
    auto buffer_size() const -> uint64_t {
        IOX_TODO();
    }
    auto receive() const -> iox::expected<iox::optional<Sample<S, Payload, UserHeader>>, SubscriberReceiveError> {
        IOX_TODO();
    }
    auto update_connections() const -> iox::expected<void, ConnectionFailure> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
