// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX_HOOFS_REPORTING_ERROR_REPORTING_MACROS_HPP
#define IOX_HOOFS_REPORTING_ERROR_REPORTING_MACROS_HPP

#include "iox/error_reporting/configuration.hpp"
#include "iox/error_reporting/error_forwarding.hpp"
#include "iox/error_reporting/types.hpp"

#include "iox/error_reporting/source_location.hpp"

// ***
// * Define public error reporting API
// ***

// NOLINTBEGIN(cppcoreguidelines-macro-usage) source location requires macros

// The following macros are statements (not expressions).
// This is important, as it enforces correct use to some degree.
// For example they cannot be used as function arguments and must be terminated with a ';'.
//
// Note: once source location becomes available without macro usage this could (and arguably should)
// be transformed into a function API.

/// @brief report error of some non-fatal kind
/// @param error error object (or code)
/// @param kind kind of error, must be non-fatal
#define IOX_REPORT(error, kind)                                                                                        \
    iox::er::forwardNonFatalError(iox::er::toError(error), kind, IOX_CURRENT_SOURCE_LOCATION, "")

/// @brief report fatal error
/// @param error error object (or code)
#define IOX_REPORT_FATAL(error)                                                                                        \
    iox::er::forwardFatalError(iox::er::toError(error), iox::er::FATAL, IOX_CURRENT_SOURCE_LOCATION, "")

/// @brief report error of some non-fatal kind if expr evaluates to true
/// @param condition boolean expression
/// @param error error object (or code)
/// @param kind kind of error, must be non-fatal
#define IOX_REPORT_IF(condition, error, kind)                                                                          \
    if (condition)                                                                                                     \
    {                                                                                                                  \
        iox::er::forwardNonFatalError(iox::er::toError(error), kind, IOX_CURRENT_SOURCE_LOCATION, #condition);         \
    }                                                                                                                  \
    [] {}() // the empty lambda forces a semicolon on the caller side

/// @brief report fatal error if expr evaluates to true
/// @param condition boolean expression
/// @param error error object (or code)
#define IOX_REPORT_FATAL_IF(condition, error)                                                                          \
    if (condition)                                                                                                     \
    {                                                                                                                  \
        iox::er::forwardFatalError(iox::er::toError(error), iox::er::FATAL, IOX_CURRENT_SOURCE_LOCATION, #condition);  \
    }                                                                                                                  \
    [] {}() // the empty lambda forces a semicolon on the caller side

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX_HOOFS_REPORTING_ERROR_REPORTING_MACROS_HPP
