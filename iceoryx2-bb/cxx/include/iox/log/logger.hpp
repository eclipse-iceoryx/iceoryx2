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

/// @todo iox-#1755 the content of this file should be customizable;
/// in order to achieve this, this file will be moved to
/// iceoryx_hoofs
///  |- customization
///      |- log
///          |- iceoryx_hoofs
///              |- log
///                  |- logger.hpp
/// the iceoryx_hoofs/customization/log path can then be set via cmake argument or toolchain file and by default will
/// point the the implementation in iceoryx_hoofs. This is similar to the iceoryx_platform customization.

#ifndef IOX_HOOFS_REPORTING_LOG_LOGGER_HPP
#define IOX_HOOFS_REPORTING_LOG_LOGGER_HPP

#include "iox/iceoryx_hoofs_deployment.hpp"
#include "iox/log/building_blocks/console_logger.hpp"
#include "iox/log/building_blocks/logger.hpp"

namespace iox
{
namespace log
{
using Logger = internal::Logger<ConsoleLogger>;
using TestingLoggerBase = internal::Logger<ConsoleLogger>;

/// @todo iox-#1755 make this a option a cmake argument and use via a compile define
/// @brief If set to true, the IOX_LOG macro will ignore the the configured log level and forward all messages to the
/// logger. This is useful in cases the default ConsoleLogger is replaced by a custom logger which does the filtering by
/// itself
/// @note This has an performance impact if set to true since the lazy evaluation of the logged data will be jimmied.
static constexpr bool IGNORE_ACTIVE_LOG_LEVEL{false};

/// @brief The minimal log level which will be compiled into the application. All log levels below this will be
/// optimized out at compile time
/// @note This is different than IGNORE_ACTIVE_LOG_LEVEL since the active log level could still be set to off at runtime
static constexpr LogLevel MINIMAL_LOG_LEVEL{build::IOX_MINIMAL_LOG_LEVEL};

} // namespace log
} // namespace iox

#endif // IOX_HOOFS_REPORTING_LOG_LOGGER_HPP
