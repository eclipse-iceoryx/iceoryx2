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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_HANDLER_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_HANDLER_HPP

#include "iox2/legacy/error_reporting/custom/default/error_handler_interface.hpp"
#include "iox2/legacy/error_reporting/source_location.hpp"
#include "iox2/legacy/error_reporting/types.hpp"

namespace iox2 {
namespace legacy {
namespace er {

/// @brief Defines the default reaction of dynamic error handling.
/// The default reaction is to do nothing apart from logging and termination on panic.
/// As this is common for all error handling of the given custom implementation, this happens in the
/// reporting API before the (polymorphic) custom behavior is invoked.
class DefaultErrorHandler : public ErrorHandlerInterface {
  public:
    DefaultErrorHandler() = default;
    ~DefaultErrorHandler() override = default;
    DefaultErrorHandler(const DefaultErrorHandler&) = delete;
    DefaultErrorHandler(DefaultErrorHandler&&) = delete;
    DefaultErrorHandler& operator=(const DefaultErrorHandler&) = delete;
    DefaultErrorHandler& operator=(DefaultErrorHandler&&) = delete;

    /// @brief Defines the reaction on panic.
    void onPanic() override {
    }

    /// @brief Defines the reaction on error.
    /// @param desc error descriptor
    void onReportError(ErrorDescriptor /* desc */) override {
    }

    /// @brief Defines the reaction on violation.
    /// @param desc error descriptor
    void onReportViolation(ErrorDescriptor /* desc */) override {
    }
};

} // namespace er
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_HANDLER_HPP
