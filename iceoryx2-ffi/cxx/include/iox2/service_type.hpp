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

#ifndef IOX2_SERVICE_TYPE_HPP
#define IOX2_SERVICE_TYPE_HPP

#include <cstdint>

namespace iox2 {
/// Defines the type of the `Service` and what kind of resources and operating system mechanisms
/// it shall use.
enum class ServiceType : uint8_t {
    /// Optimized for inter-thread communication does not support inter-process communication.
    Local,
    /// Optimized for inter-process communication.
    Ipc
};
} // namespace iox2

#endif
