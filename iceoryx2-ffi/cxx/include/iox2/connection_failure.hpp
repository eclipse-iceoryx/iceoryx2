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

#ifndef IOX2_CONNECTION_FAILURE_HPP
#define IOX2_CONNECTION_FAILURE_HPP

#include <cstdint>

namespace iox2 {
/// Describes the errors that can occur when a connection between two endpoints (ports) is
/// established
enum class ConnectionFailure : uint8_t {
    /// Failures when creating the connection
    FailedToEstablishConnection,

    /// Failures when mapping the corresponding data segment
    UnableToMapSendersDataSegment
};

} // namespace iox2

#endif
