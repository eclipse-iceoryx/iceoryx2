// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_REPORTING_IMPL_HPP
#define IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_REPORTING_IMPL_HPP

#include "iox/error_reporting/error_kind.hpp"
#include "iox/error_reporting/error_logging.hpp"
#include "iox/error_reporting/source_location.hpp"
#include "iox/error_reporting/types.hpp"
#include "iox/error_reporting/violation.hpp"

#include "iox/error_reporting/custom/default/error_handler.hpp"
#include "iox/error_reporting/custom/error_kind.hpp"

namespace iox
{
namespace er
{

// The static reporting interface that must be defined to at least do nothing.
// It should provide a noreturn specification for panic (but since it be assumed that the custom
// code enforces this, it is enforced at the (non-custom) forwarding level.

// Here, the implementation redirects to the polymorphic handler interface.
// This adds an additional indirection but is required for switching handlers
// during operation.
// This is used for testing but must be done while no errors are reported concurrently,
// otherwise error notifations could be lost.

// The logging can be extended in the future.

// Custom panic
[[noreturn]] inline void panic()
{
    auto& h = ErrorHandler::get();
    h.onPanic();
    abort();
}

// Custom panic with location
[[noreturn]] inline void panic(const SourceLocation& location)
{
    IOX_ERROR_INTERNAL_LOG_PANIC(location, "[PANIC]");
    panic();
}

// Custom panic with location and message
// note that Message is generic as the logger technically accepts more general loggable constructs
// beyond const char*
template <class Message>
// NOLINTNEXTLINE(cppcoreguidelines-missing-std-forward) false positive, this is used as a universal reference
[[noreturn]] inline void panic(const SourceLocation& location, Message&& msg)
{
    IOX_ERROR_INTERNAL_LOG_PANIC(location, "[PANIC] " << msg);
    panic();
}

namespace detail
{
inline log::LogStream& logStringifiedCondition(log::LogStream& stream, const char* stringifiedCondition)
{
    if (stringifiedCondition != nullptr && strnlen(stringifiedCondition, 1) != 0)
    {
        stream << "Conditiopn: \"" << stringifiedCondition << "\" ";
    }
    return stream;
}
} // namespace detail

// Report any error, general version.
template <class Kind, class Error>
inline void report(const SourceLocation& location, Kind, const Error& error, const char* stringifiedCondition)
{
    auto code = toCode(error);
    auto module = toModule(error);
    auto moduleName = toModuleName(error);
    auto errorName = toErrorName(error);

    IOX_ERROR_INTERNAL_LOG(location,
                           [stringifiedCondition](auto& stream) -> auto& {
                               return detail::logStringifiedCondition(stream, stringifiedCondition);
                           } << "["
                             << errorName << " (code = " << code.value << ")] in module [" << moduleName
                             << " (id = " << module.value << ")]");
    auto& h = ErrorHandler::get();
    h.onReportError(ErrorDescriptor(location, code, module));
}

// Report any error, specialization for specific types overrides the general version.
// Any behaviour for specific error types (and kinds) has to be defined like this.
//
// Here the logging is subtly different and does not easily allow to factor out common parts.

template <class Error>
inline void
report(const SourceLocation& location, iox::er::FatalKind kind, const Error& error, const char* stringifiedCondition)
{
    auto code = toCode(error);
    auto module = toModule(error);
    auto moduleName = toModuleName(error);
    auto errorName = toErrorName(error);

    IOX_ERROR_INTERNAL_LOG_FATAL(location,
                                 [stringifiedCondition](auto& stream) -> auto& {
                                     return detail::logStringifiedCondition(stream, stringifiedCondition);
                                 } << "["
                                   << kind.name << "] [" << errorName << " (code = " << code.value << ")] in module ["
                                   << moduleName << " (id = " << module.value << ")]");
    auto& h = ErrorHandler::get();
    h.onReportError(ErrorDescriptor(location, code, module));
}

namespace detail
{
template <class Kind, class Error>
inline void
// NOLINTNEXTLINE(readability-function-size) Not used directly but via a macro which hides the number of parameter away
report(const SourceLocation& location, Kind kind, const Error& error, const char* stringifiedCondition)
{
    auto code = toCode(error);
    auto module = toModule(error);
    IOX_ERROR_INTERNAL_LOG_FATAL(location,
                                 [stringifiedCondition](auto& stream) -> auto& {
                                     return detail::logStringifiedCondition(stream, stringifiedCondition);
                                 } << "["
                                   << kind.name << "]");
    auto& h = ErrorHandler::get();
    h.onReportViolation(ErrorDescriptor(location, code, module));
}

template <class Kind, class Error, class Message>
inline void
// NOLINTNEXTLINE(readability-function-size) Not used directly but via a macro which hides the number of parameter away
report(const SourceLocation& location, Kind kind, const Error& error, const char* stringifiedCondition, Message&& msg)
{
    auto code = toCode(error);
    auto module = toModule(error);
    IOX_ERROR_INTERNAL_LOG_FATAL(location,
                                 [stringifiedCondition](auto& stream) -> auto& {
                                     return detail::logStringifiedCondition(stream, stringifiedCondition);
                                 } << "["
                                   << kind.name << "] " << std::forward<Message>(msg));
    auto& h = ErrorHandler::get();
    h.onReportViolation(ErrorDescriptor(location, code, module));
}
} // namespace detail

template <class Error>
inline void report(const SourceLocation& location,
                   iox::er::AssertViolationKind kind,
                   const Error& error,
                   const char* stringifiedCondition)
{
    detail::report(location, kind, error, stringifiedCondition);
}

template <class Error>
inline void report(const SourceLocation& location,
                   iox::er::EnforceViolationKind kind,
                   const Error& error,
                   const char* stringifiedCondition)
{
    detail::report(location, kind, error, stringifiedCondition);
}

template <class Error, class Message>
// NOLINTNEXTLINE(readability-function-size) Not used directly but via a macro which hides the number of parameter away
inline void report(const SourceLocation& location,
                   iox::er::AssertViolationKind kind,
                   const Error& error,
                   const char* stringifiedCondition,
                   Message&& msg)
{
    detail::report(location, kind, error, stringifiedCondition, std::forward<Message>(msg));
}

template <class Error, class Message>
// NOLINTNEXTLINE(readability-function-size) Not used directly but via a macro which hides the number of parameter away
inline void report(const SourceLocation& location,
                   iox::er::EnforceViolationKind kind,
                   const Error& error,
                   const char* stringifiedCondition,
                   Message&& msg)
{
    detail::report(location, kind, error, stringifiedCondition, std::forward<Message>(msg));
}

} // namespace er
} // namespace iox

#endif // IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_REPORTING_IMPL_HPP
