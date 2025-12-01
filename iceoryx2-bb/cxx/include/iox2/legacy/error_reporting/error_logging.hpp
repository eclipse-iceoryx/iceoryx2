// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_LOGGING_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_LOGGING_HPP

#include "iox2/legacy/error_reporting/source_location.hpp"
#include "iox2/legacy/logging.hpp"

// with a log stream interface this could be done with functions, not macros
// NOLINTBEGIN(cppcoreguidelines-macro-usage, bugprone-macro-parentheses) macros are required without logstream interface

/// @brief Log the location of an error.
/// @param location the location of the error
/// @param msg_stream is the log message stream; multiple items can be logged by using the '<<' operator
#define IOX2_ERROR_INTERNAL_LOG(location, msg_stream)                                                                  \
    IOX2_LOG_INTERNAL(location.file,                                                                                   \
                      location.line,                                                                                   \
                      location.function,                                                                               \
                      iox2::legacy::log::LogLevel::Error,                                                              \
                      location.file << ":" << location.line << " " << msg_stream)

/// @brief Log the location of a fatal error.
/// @param location the location of the error
/// @param msg_stream is the log message stream; multiple items can be logged by using the '<<' operator
#define IOX2_ERROR_INTERNAL_LOG_FATAL(location, msg_stream)                                                            \
    IOX2_LOG_INTERNAL(location.file,                                                                                   \
                      location.line,                                                                                   \
                      location.function,                                                                               \
                      iox2::legacy::log::LogLevel::Fatal,                                                              \
                      location.file << ":" << location.line << " " << msg_stream)

/// @brief Log a panic invocation.
/// @param location the location of the panic invocation.
/// @param msg_stream is the log message stream; multiple items can be logged by using the '<<' operator
#define IOX2_ERROR_INTERNAL_LOG_PANIC(location, msg_stream) IOX2_ERROR_INTERNAL_LOG_FATAL(location, msg_stream)

// NOLINTEND(cppcoreguidelines-macro-usage, bugprone-macro-parentheses)

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_ERROR_LOGGING_HPP
