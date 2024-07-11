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

#ifndef IOX2_NOTIFIER_HPP_
#define IOX2_NOTIFIER_HPP_

#include "event_id.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "service_type.hpp"
#include "unique_port_id.hpp"

namespace iox2 {
enum class NotifierCreateError {
    /// The maximum amount of [`Notifier`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more
    /// [`Notifier`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedNotifiers,
};

enum class NotifierNotifyError {
    /// A [`Notifier::notify_with_custom_event_id()`] was called and the
    /// provided [`EventId`]
    /// is greater than the maximum supported [`EventId`] by the
    /// [`Service`](crate::service::Service)
    EventIdOutOfBounds,
};

template <ServiceType S>
class Notifier {
  public:
    UniqueNotifierId id() const {
        IOX_TODO();
    }
    iox::expected<uint64_t, NotifierNotifyError> notify() const {
        IOX_TODO();
    }
    iox::expected<uint64_t, NotifierNotifyError> notify_with_custom_event_id(const EventId id) const {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
