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

#ifndef IOX2_WAIT_EVENT_HPP
#define IOX2_WAIT_EVENT_HPP

#include <cstdint>

namespace iox2 {
/// Defines all possible events that can occur during [`Node::wait()`].
enum class WaitEvent : uint8_t {
    /// The timeout passed.
    Tick,
    /// SIGTERM signal was received
    TerminationRequest,
    /// SIGINT signal was received
    Interrupt,
    /// An event notification was received
    Notification
};
} // namespace iox2

#endif
