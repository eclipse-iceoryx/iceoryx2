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

#ifndef IOX2_DEGRADATION_ACTION_HPP
#define IOX2_DEGRADATION_ACTION_HPP

#include <cstdint>

namespace iox2 {
/// Defines the action that shall be take when an degradation is detected. This can happen when
/// data cannot be delivered, or when the system is corrupted and files are modified by
/// non-iceoryx2 instances. Is used as return value of the [`DegradationHandler`] to define a
/// custom behavior.
enum class DegradationAction : uint8_t {
    /// Ignore the degradation completely
    Ignore,
    /// Print out a warning as soon as the degradation is detected
    Warn,
    /// Returns a failure in the function the degradation was detected
    DegradeAndFail,
};
} // namespace iox2

#endif // IOX2_DEGRADATION_ACTION_HPP
