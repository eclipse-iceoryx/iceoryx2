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

#ifndef IOX2_LISTENER_HPP
#define IOX2_LISTENER_HPP

#include "event_id.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox/duration.hpp"
#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox/optional.hpp"
#include "service_type.hpp"
#include "unique_port_id.hpp"

#include <cstdint>

namespace iox2 {
enum class ListenerCreateError : uint8_t {
    /// The maximum amount of [`Listener`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more
    /// [`Listener`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedListeners,
    /// An underlying resource of the [`Service`](crate::service::Service) could
    /// not be created
    ResourceCreationFailed,
};

enum class ListenerWaitError : uint8_t {
    ContractViolation,
    InternalFailure,
    InterruptSignal,
};

template <ServiceType>
class Listener {
  public:
    auto id() const -> UniqueListenerId {
        IOX_TODO();
    }

    auto try_wait_all(const iox::function<void(EventId)>& callback) -> iox::expected<void, ListenerWaitError> {
        IOX_TODO();
    }
    auto timed_wait_all(const iox::function<void(EventId)>& callback,
                        const iox::units::Duration& timeout) -> iox::expected<void, ListenerWaitError> {
        IOX_TODO();
    }
    auto blocking_wait_all(const iox::function<void(EventId)>& callback) -> iox::expected<void, ListenerWaitError> {
        IOX_TODO();
    }

    auto try_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
        IOX_TODO();
    }
    auto
    timed_wait_one(const iox::units::Duration& timeout) -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
        IOX_TODO();
    }
    auto blocking_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
