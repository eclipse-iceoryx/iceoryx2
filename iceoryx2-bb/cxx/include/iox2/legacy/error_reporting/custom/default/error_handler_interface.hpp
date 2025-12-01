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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_INTERFACE_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_INTERFACE_HPP

#include "iox2/legacy/error_reporting/source_location.hpp"
#include "iox2/legacy/error_reporting/types.hpp"
#include "iox2/legacy/error_reporting/violation.hpp"

namespace iox2 {
namespace legacy {
namespace er {
/// @brief Contains all required information about the error.
/// Can be extended as needed without breaking the interface.
/// @note We either need this, something like std::any or a class hierarchy for runtime polymorphism.
/// The actual error type must be erased in some way.
struct ErrorDescriptor {
    constexpr ErrorDescriptor(const SourceLocation& location,
                              const ErrorCode& code,
                              const ModuleId& module = ModuleId())
        : location(location)
        , code(code)
        , module(module) {
    }

    SourceLocation location;
    ErrorCode code;
    ModuleId module;
};

/// @brief Defines the dynamic error handling interface (i.e. changeable at runtime).
// NOLINTJUSTIFICATION abstract interface
// NOLINTNEXTLINE(cppcoreguidelines-special-member-functions, hicpp-special-member-functions)
class ErrorHandlerInterface {
  public:
    virtual ~ErrorHandlerInterface() = default;

    /// @brief Defines the reaction on panic.
    virtual void onPanic() = 0;

    /// @brief Defines the reaction on error.
    /// @param desc error descriptor
    virtual void onReportError(ErrorDescriptor desc) = 0;

    /// @brief Defines the reaction on violation (a bug in the code)
    /// @param desc error descriptor
    virtual void onReportViolation(ErrorDescriptor desc) = 0;
};

} // namespace er
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_INTERFACE_HPP
