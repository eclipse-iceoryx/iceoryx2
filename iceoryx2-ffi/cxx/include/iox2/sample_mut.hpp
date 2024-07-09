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
#ifndef IOX2_SAMPLE_MUT_HPP_
#define IOX2_SAMPLE_MUT_HPP_

#include <cstdint>

#include "header_publish_subscribe.hpp"
#include "iox/expected.hpp"
#include "service_type.hpp"

namespace iox2 {
/// Failure that can be emitted when a [`SampleMut`] is sent via
/// [`SampleMut::send()`].
enum PublisherSendError {
    /// [`SampleMut::send()`] was called but the corresponding [`Publisher`]
    /// went already out of
    /// scope.
    ConnectionBrokenSincePublisherNoLongerExists,
    /// A connection between a
    /// [`Subscriber`](crate::port::subscriber::Subscriber) and a
    /// [`Publisher`] is corrupted.
    ConnectionCorrupted,
    /// A failure occurred while acquiring memory for the payload
    LoanError,
    /// A failure occurred while establishing a connection to a
    /// [`Subscriber`](crate::port::subscriber::Subscriber)
    ConnectionError,
};

template <ServiceType S, typename Payload, typename UserHeader>
class SampleMut {
   public:
    const HeaderPublishSubscribe& header() const {}
    const UserHeader& user_header() const {}
    UserHeader& user_header_mut() {}
    const Payload& payload() const {}
    Payload& payload_mut() {}

    static iox::expected<uint64_t, PublisherSendError> send(
        SampleMut&& sample) {}
};
}  // namespace iox2

#endif
