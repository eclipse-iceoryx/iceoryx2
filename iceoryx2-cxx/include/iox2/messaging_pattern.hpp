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

#ifndef IOX2_MESSAGING_PATTERN_HPP
#define IOX2_MESSAGING_PATTERN_HPP

#include <cstdint>
#include <ostream>

namespace iox2 {
enum class MessagingPattern : uint8_t {
    /// Unidirectional communication pattern where the
    /// [`Publisher`](crate::port::publisher::Publisher) sends arbitrary data to
    /// the
    /// [`Subscriber`](crate::port::subscriber::Subscriber)
    PublishSubscribe = 0,

    /// Unidirectional communication pattern where the
    /// [`Notifier`](crate::port::notifier::Notifier)
    /// sends signals/events to the
    /// [`Listener`](crate::port::listener::Listener) which has the
    /// ability to sleep until a signal/event arrives.
    /// Building block to realize push-notifications.
    Event,

    /// Bidirectional communication pattern where the
    /// [`Client`](crate::port::client::Client) sends arbitrary data in form of requests to the
    /// [`Server`](crate::port::server::Server) and receives a stream of responses.
    RequestResponse,
};
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::MessagingPattern& value) -> std::ostream&;

#endif
