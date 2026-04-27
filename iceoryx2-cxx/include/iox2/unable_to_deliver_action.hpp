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

#ifndef IOX2_UNABLE_TO_DELIVER_ACTION_HPP
#define IOX2_UNABLE_TO_DELIVER_ACTION_HPP

#include <cstdint>

namespace iox2 {
/// Defines the action that shall be take when data cannot be delivered.
enum class UnableToDeliverAction : uint8_t {
    /// Use an action which is derived from the `UnableToDeliverStrategy`
    FollowUnableToDeliveryStrategy,
    /// Retry to send and invoke the handler again, if sending does not succeed
    Retry,
    /// Discard the data for the receiver which cause the incident and continue
    /// to deliver the data to the remaining receivers
    DiscardData,
    /// Discard the data for the receiver which caused the incident, continue
    /// to deliver the data to the remaining receivers;
    /// return with an error if the data was not delivered to all receivers
    DiscardDataAndFail,
};
} // namespace iox2

#endif // IOX2_UNABLE_TO_DELIVER_ACTION_HPP
