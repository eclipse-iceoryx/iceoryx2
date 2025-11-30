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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_MACROS_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_MACROS_HPP

#include "iox2/legacy/error_reporting/configuration.hpp"
#include "iox2/legacy/error_reporting/error_forwarding.hpp"
#include "iox2/legacy/error_reporting/types.hpp"

#include "iox2/legacy/error_reporting/source_location.hpp"

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
#define IOX2_REPORT(error, kind)                                                                                       \
    iox2::legacy::er::forwardNonFatalError(iox2::legacy::er::toError(error), kind, IOX2_CURRENT_SOURCE_LOCATION, "")

/// @brief report fatal error
/// @param error error object (or code)
#define IOX2_REPORT_FATAL(error)                                                                                       \
    iox2::legacy::er::forwardFatalError(                                                                               \
        iox2::legacy::er::toError(error), iox2::legacy::er::FATAL, IOX2_CURRENT_SOURCE_LOCATION, "")

/// @brief report error of some non-fatal kind if expr evaluates to true
/// @param condition boolean expression
/// @param error error object (or code)
/// @param kind kind of error, must be non-fatal
#define IOX2_REPORT_IF(condition, error, kind)                                                                         \
    if (condition) {                                                                                                   \
        iox2::legacy::er::forwardNonFatalError(                                                                        \
            iox2::legacy::er::toError(error), kind, IOX2_CURRENT_SOURCE_LOCATION, #condition);                         \
    }                                                                                                                  \
    [] { }() // the empty lambda forces a semicolon on the caller side

/// @brief report fatal error if expr evaluates to true
/// @param condition boolean expression
/// @param error error object (or code)
#define IOX2_REPORT_FATAL_IF(condition, error)                                                                         \
    if (condition) {                                                                                                   \
        iox2::legacy::er::forwardFatalError(                                                                           \
            iox2::legacy::er::toError(error), iox2::legacy::er::FATAL, IOX2_CURRENT_SOURCE_LOCATION, #condition);      \
    }                                                                                                                  \
    [] { }() // the empty lambda forces a semicolon on the caller side

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_MACROS_HPP
