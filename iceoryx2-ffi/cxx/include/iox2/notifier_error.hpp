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

#ifndef IOX2_NOTIFIER_ERROR_HPP
#define IOX2_NOTIFIER_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Failures that can occur when a new [`Notifier`] is created with the
/// [`PortFactoryNotifier`].
enum class NotifierCreateError : uint8_t {
    /// The maximum amount of [`Notifier`]s that can connect to a
    /// [`Service`] is defined in [`Config`]. When this is exceeded no more
    /// [`Notifier`]s can be created for a specific [`Service`].
    ExceedsMaxSupportedNotifiers,
    /// Caused by a failure when instantiating a [`ArcSyncPolicy`] defined in the
    /// [`Service`] as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
};

/// Defines the failures that can occur while a [`Notifier::notify()`] call.
enum class NotifierNotifyError : uint8_t {
    /// A [`Notifier::notify_with_custom_event_id()`] was called and the
    /// provided [`EventId`] is greater than the maximum supported [`EventId`] by the
    /// [`Service`]
    EventIdOutOfBounds,
    /// The notification was delivered to all [`Listener`] ports
    /// but the deadline contract, the maximum time span between two notifications, of the
    /// [`Service`] was violated.
    MissedDeadline,
    /// The notification was delivered but the elapsed system time could not be acquired.
    /// Therefore, it is unknown if the deadline was missed or not.
    UnableToAcquireElapsedTime,
};

} // namespace iox2

#endif
