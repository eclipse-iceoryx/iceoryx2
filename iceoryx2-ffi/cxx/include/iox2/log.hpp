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

#ifndef IOX2_LOG_HPP
#define IOX2_LOG_HPP

#include "iox2/log_level.hpp"

namespace iox2 {

/// Sets the global log level for the application
auto set_log_level(LogLevel level) -> void;

/// Returns the current global log level of the application
auto get_log_level() -> LogLevel;

} // namespace iox2

#endif
