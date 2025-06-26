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

#ifndef IOX2_LISTENER_ERROR_HPP
#define IOX2_LISTENER_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Defines the failures that can occur when a [`Listener`] is created with the
/// [`PortFactoryListener`].
enum class ListenerCreateError : uint8_t {
    /// The maximum amount of [`Listener`]s that can connect to a
    /// [`Service`] is defined in [`Config`]. When this is exceeded no more
    /// [`Listener`]s can be created for a specific [`Service`].
    ExceedsMaxSupportedListeners,
    /// An underlying resource of the [`Service`] could
    /// not be created
    ResourceCreationFailed,
    /// Caused by a failure when instantiating a [`ArcSyncPolicy`] defined in the
    /// [`Service`] as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
};

/// Defines failures that can occur while waiting for a notification from a
/// [`Notifier`] on a [`Listener`]
enum class ListenerWaitError : uint8_t {
    /// The notification payload did not satisfy the underlying contract.
    ContractViolation,
    /// An interrupt signal was raised while waiting for a notification.
    InterruptSignal,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    InternalFailure,
};

} // namespace iox2

#endif
