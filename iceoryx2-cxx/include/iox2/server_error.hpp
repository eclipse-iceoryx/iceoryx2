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

#ifndef IOX2_SERVER_ERROR_HPP
#define IOX2_SERVER_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Defines a failure that can occur when a [`Server`] is created with
/// [`PortFactoryServer`].
enum class ServerCreateError : uint8_t {
    /// The maximum amount of [`Server`]s supported by the [`Service`]
    /// is already connected.
    ExceedsMaxSupportedServers,
    /// The datasegment in which the payload of the [`Server`] is stored, could not be created.
    UnableToCreateDataSegment,
    /// Caused by a failure when instantiating a [`ArcSyncPolicy`] defined in the
    /// [`Service`] as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
};
} // namespace iox2
#endif
