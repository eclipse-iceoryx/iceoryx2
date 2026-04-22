// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#ifndef IOX2_DEGRADATION_HANDLER_ENUMS_HPP
#define IOX2_DEGRADATION_HANDLER_ENUMS_HPP

#include <cstdint>

// NOTE: separate header for the enums to prevent circular dependencies between the headers

namespace iox2 {
enum class DegradationAction : uint8_t {
    /// Perform the default action
    Default,
    /// Ignore the degradation completely
    Ignore,
    /// Performs whatever is necessary to discard the degradation
    Discard,
    /// Retries the action that caused the degradation
    Retry,
    /// Blocks until the cause of the degradation disappeared
    Block,
    /// Print out a warning as soon as the degradation is detected
    Warn,
    /// Returns a failure in the function the degradation was detected
    Fail,
};

/// Defines the cause of a degradation and is a parameter of the [`DegradationCallback`].
enum class DegradationCause : uint8_t {
    /// Connection could not be established
    FailedToEstablishConnection,
    /// Connection is corrupted
    ConnectionCorrupted,
    /// Data could not be delivered
    UnableToDeliverData,
    /// The [`DegradationAction`] used by the [`DegradationCallback`] was invalid for the given [`DegradationCause`].
    /// The function will return with an error after the invocation of the [`DegradationCallback`].
    InvalidDegradationAction,
};
} // namespace iox2

#endif // IOX2_DEGRADATION_HANDLER_ENUMS_HPP
