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

#include "iox2/log.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {

auto set_log_level(LogLevel level) -> void {
    iox2_set_log_level(iox::into<iox2_log_level_e>(level));
}

auto get_log_level() -> LogLevel {
    return LogLevel(iox2_get_log_level());
}

} // namespace iox2
