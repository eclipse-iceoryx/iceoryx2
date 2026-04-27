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

#ifndef IOX2_UNABLE_TO_DELIVER_STRATEGY_HPP
#define IOX2_UNABLE_TO_DELIVER_STRATEGY_HPP

#include <cstdint>

namespace iox2 {
/// Defines the strategy a sender shall pursue when the buffer of the receiver is full
/// and the service does not overflow.
enum class UnableToDeliverStrategy : uint8_t {
    /// Retries until the receiver has consumed some
    /// data from the full buffer and there is space again
    RetryUntilDelivered,
    /// Do not deliver the data to receiver with a full buffer
    DiscardData,
};
} // namespace iox2

#endif
