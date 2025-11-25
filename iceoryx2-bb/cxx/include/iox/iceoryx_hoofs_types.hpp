// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_PRIMITIVES_ICEORYX_HOOFS_TYPES_HPP
#define IOX_HOOFS_PRIMITIVES_ICEORYX_HOOFS_TYPES_HPP

/// @note since this file will be included by many other files, it should not include other header except
/// iceoryx_platform or STL header

#include "iceoryx_platform/logging.hpp"

#include <cstdint>

namespace iox
{
using byte = uint8_t;

// AXIVION Next Construct AutosarC++19_03-M2.10.1 : log is a sensible namespace for a logger; furthermore it is in the
// iox namespace and when used as function the compiler will complain
namespace log
{
/// @brief This enum defines the log levels used for logging.
enum class LogLevel : uint8_t
{
    Off = IceoryxPlatformLogLevel::IOX_PLATFORM_LOG_LEVEL_OFF,
    Fatal = IceoryxPlatformLogLevel::IOX_PLATFORM_LOG_LEVEL_FATAL,
    Error = IceoryxPlatformLogLevel::IOX_PLATFORM_LOG_LEVEL_ERROR,
    Warn = IceoryxPlatformLogLevel::IOX_PLATFORM_LOG_LEVEL_WARN,
    Info = IceoryxPlatformLogLevel::IOX_PLATFORM_LOG_LEVEL_INFO,
    Debug = IceoryxPlatformLogLevel::IOX_PLATFORM_LOG_LEVEL_DEBUG,
    Trace = IceoryxPlatformLogLevel::IOX_PLATFORM_LOG_LEVEL_TRACE,
};

/// @brief converts LogLevel into a string literal
/// @param[in] value the LogLevel to convert
/// @return string literal of the LogLevel value
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : This function return a string literal
// which corresponds to a const char *
constexpr const char* asStringLiteral(const LogLevel value) noexcept;

} // namespace log
} // namespace iox

#include "iox/detail/iceoryx_hoofs_types.inl"

#endif // IOX_HOOFS_PRIMITIVES_ICEORYX_HOOFS_TYPES_HPP
