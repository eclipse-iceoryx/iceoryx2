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
#include "iox/into.hpp"
#include "iox/optional.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
namespace {
//NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables): it is in an anonymous namespace and therefore only accessible in this compilation unit
iox::optional<Log*> global_logger = iox::nullopt;

void internal_log_callback(iox2_log_level_e log_level, const char* origin, const char* message) {
    (*global_logger)->log(iox::into<LogLevel>(static_cast<int>(log_level)), origin, message);
}

} // namespace

auto set_logger(Log& logger) -> bool {
    auto success = iox2_set_logger(internal_log_callback);
    if (success) {
        global_logger.emplace(&logger);
    }
    return success;
}

void log(LogLevel log_level, const char* origin, const char* message) {
    iox2_log(iox::into<iox2_log_level_e>(log_level), origin, message);
}

auto set_log_level_from_env_or_default() -> void {
    iox2_set_log_level_from_env_or_default();
}

auto set_log_level_from_env_or(LogLevel level) -> void {
    iox2_set_log_level_from_env_or(iox::into<iox2_log_level_e>(level));
}

auto set_log_level(LogLevel level) -> void {
    iox2_set_log_level(iox::into<iox2_log_level_e>(level));
}

auto get_log_level() -> LogLevel {
    return LogLevel(iox2_get_log_level());
}

auto use_console_logger() -> bool {
    return iox2_use_console_logger();
}

auto use_file_logger(const char* log_file) -> bool {
    return iox2_use_file_logger(log_file);
}
} // namespace iox2
