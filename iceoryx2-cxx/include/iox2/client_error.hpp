// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#ifndef IOX2_CLIENT_ERROR_HPP
#define IOX2_CLIENT_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Defines a failure that can occur when a [`Client`] is created with
/// [`PortFactoryClient`].
enum class ClientCreateError : uint8_t {
    /// The datasegment in which the payload of the [`Client`] is stored, could not be created.
    UnableToCreateDataSegment,
    /// The maximum amount of [`Client`]s that can connect to a
    /// [`Service`] is
    /// defined in [`Config`]. When this is exceeded no more [`Client`]s
    /// can be created for a specific [`Service`].
    ExceedsMaxSupportedClients,
    /// Caused by a failure when instantiating a [`ArcSyncPolicy`] defined in the
    /// [`Service`](crate::service::Service) as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
};
} // namespace iox2
#endif
