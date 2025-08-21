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

#ifndef IOX2_SIGNAL_HANDLING_MODE_HPP
#define IOX2_SIGNAL_HANDLING_MODE_HPP

#include <cstdint>

namespace iox2 {
/// Defines how signals are handled by constructs that might register a custom
/// [`SignalHandler`]
enum class SignalHandlingMode : uint8_t {
    /// The signals `SIGINT` and `SIGTERM` are registered and handled. If such a Signal is received
    /// the user will be notified.
    HandleTerminationRequests,
    /// No signal handler will be registered.
    Disabled,
};
} // namespace iox2

#endif
