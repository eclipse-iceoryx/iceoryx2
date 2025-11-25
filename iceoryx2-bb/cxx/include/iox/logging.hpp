// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX_HOOFS_REPORTING_LOGGING_HPP
#define IOX_HOOFS_REPORTING_LOGGING_HPP

#include "iox/log/logstream.hpp"

namespace iox
{
namespace log
{
namespace internal
{
/// @brief Convenience function for the IOX_LOG_INTERNAL macro
inline bool isLogLevelActive(LogLevel logLevel) noexcept
{
    // AXIVION Next Construct FaultDetection-DeadBranches this is a configurable compile time option to be able to
    // optimize the logger call away during compilation and intended
    // AXIVION Next Construct AutosarC++19_03-M0.1.2 see justification for FaultDetection-DeadBranches
    // AXIVION Next Construct AutosarC++19_03-M0.1.9 see justification for FaultDetection-DeadBranches
    // AXIVION Next Construct AutosarC++19_03-M5.14.1 getLogLevel is a static method without side effects
    return ((logLevel) <= MINIMAL_LOG_LEVEL) && (IGNORE_ACTIVE_LOG_LEVEL || ((logLevel) <= log::Logger::getLogLevel()));
}
} // namespace internal
} // namespace log
} // namespace iox

/// @brief Only for internal usage
// AXIVION Next Construct AutosarC++19_03-A16.0.1 cannot be realized with templates or constexpr functions due to the
// intended lazy evaluation technique with the if statement
// NOLINTBEGIN(cppcoreguidelines-macro-usage)
// NOLINTBEGIN(bugprone-macro-parentheses) 'msg_stream' cannot be wrapped in parentheses due to the '<<' operator
#define IOX_LOG_INTERNAL(file, line, function, level, msg_stream)                                                      \
    if (iox::log::internal::isLogLevelActive(level))                                                                   \
    {                                                                                                                  \
        iox::log::LogStream(file, line, function, level).self() << msg_stream;                                         \
    }                                                                                                                  \
    [] {}() // the empty lambda forces a semicolon on the caller side
            // NOLINTEND(bugprone-macro-parentheses)

/// @brief Macro for logging
/// @param[in] level is the log level to be used for the log message
/// @param[in] msg_stream is the log message stream; multiple items can be logged by using the '<<' operator
/// @code
///     IOX_LOG(Info, "Hello" << " World");
/// @endcode
// AXIVION Next Construct AutosarC++19_03-A16.0.1 needed for source code location, safely wrapped in macro
// AXIVION Next Construct AutosarC++19_03-M16.0.6 brackets around macro parameter would lead to compile time failures in this case
// NOLINTJUSTIFICATION __PRETTY_FUNCTION__ would work better in lambdas but especially with
// templates the resulting string is too large; we also get the file name and the line of the invocation which should be
// sufficient for debugging
// NOLINTBEGIN(bugprone-lambda-function-name)
#define IOX_LOG(level, msg_stream)                                                                                     \
    IOX_LOG_INTERNAL(__FILE__, __LINE__, static_cast<const char*>(__FUNCTION__), iox::log::LogLevel::level, msg_stream)
// NOLINTEND(bugprone-lambda-function-name)

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX_HOOFS_REPORTING_LOGGING_HPP
