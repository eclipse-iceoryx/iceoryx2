// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_ASSERTIONS_HPP
#define IOX2_BB_REPORTING_ASSERTIONS_HPP

#include "iox2/legacy/error_reporting/configuration.hpp"
#include "iox2/legacy/error_reporting/error_forwarding.hpp"

#include "iox2/legacy/error_reporting/source_location.hpp"

// ***
// * Define public assertion API
// ***

// NOLINTBEGIN(cppcoreguidelines-macro-usage) source location requires macros

// The following macros are statements (not expressions).
// This is important, as it enforces correct use to some degree.
// For example they cannot be used as function arguments and must be terminated with a ';'.
//
// Note: once source location becomes available without macro usage this could (and arguably should)
// be transformed into a function API.

/// @brief calls panic handler and does not return
/// @param message message to be forwarded
/// @note could actually throw if desired without breaking control flow asssumptions
#define IOX2_PANIC(message) iox2::legacy::er::forwardPanic(IOX2_CURRENT_SOURCE_LOCATION, message)

//************************************************************************************************
//* For documentation of intent, defensive programming and debugging
//*
//* There are no error codes/errors required here on purpose, as it would make the use cumbersome.
//* Instead a special internal error type is used.
//************************************************************************************************

/// @brief only for debug builds: report fatal assert violation if expression evaluates to false
/// @note for conditions that should not happen with correct use
/// @param condition boolean expression that must hold
/// @param message message to be forwarded in case of violation
#define IOX2_ASSERT(condition, message)                                                                                \
    if (iox2::legacy::er::Configuration::CHECK_ASSERT && !(condition)) {                                               \
        iox2::legacy::er::forwardFatalError(iox2::legacy::er::Violation::createAssertViolation(),                      \
                                            iox2::legacy::er::ASSERT_VIOLATION,                                        \
                                            IOX2_CURRENT_SOURCE_LOCATION,                                              \
                                            #condition,                                                                \
                                            message);                                                                  \
    }                                                                                                                  \
    []() -> void { }() // the empty lambda forces a semicolon on the caller side

/// @brief report fatal enforce violation if expression evaluates to false
/// @note for conditions that may actually happen during correct use
/// @param condition boolean expression that must hold
/// @param message message to be forwarded in case of violation
#define IOX2_ENFORCE(condition, message)                                                                               \
    if (!(condition)) {                                                                                                \
        iox2::legacy::er::forwardFatalError(iox2::legacy::er::Violation::createEnforceViolation(),                     \
                                            iox2::legacy::er::ENFORCE_VIOLATION,                                       \
                                            IOX2_CURRENT_SOURCE_LOCATION,                                              \
                                            #condition,                                                                \
                                            message);                                                                  \
    }                                                                                                                  \
    []() -> void { }() // the empty lambda forces a semicolon on the caller side

/// @brief panic if control flow reaches this code at runtime
#define IOX2_UNREACHABLE()                                                                                             \
    iox2::legacy::er::detail::unreachable_wrapped<void, void>(IOX2_CURRENT_SOURCE_LOCATION,                            \
                                                              "Reached code that was supposed to be unreachable.")

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX2_BB_REPORTING_ASSERTIONS_HPP
