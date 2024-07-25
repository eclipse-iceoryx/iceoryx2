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

#ifndef IOX2_SUBSCRIBER_ERROR_HPP
#define IOX2_SUBSCRIBER_ERROR_HPP

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
} // namespace iox2

#endif
