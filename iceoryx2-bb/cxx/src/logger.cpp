
// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#include "iox/log/building_blocks/logger.hpp"
#include "iox/iceoryx_hoofs_types.hpp"

#include <cstdio>
#include <cstdlib>

namespace iox
{
namespace log
{
LogLevel logLevelFromEnvOr(const LogLevel logLevel) noexcept
{
    auto specifiedLogLevel = logLevel;

    // AXIVION Next Construct AutosarC++19_03-M18.0.3 : Use of getenv is allowed in MISRA amendment#6312
    // JUSTIFICATION getenv is required for the functionality of this function; see also declaration in header
    // NOLINTNEXTLINE(concurrency-mt-unsafe)
    if (const auto* logLevelString = std::getenv("IOX_LOG_LEVEL"))
    {
        if (equalStrings(logLevelString, "off"))
        {
            specifiedLogLevel = LogLevel::Off;
        }
        else if (equalStrings(logLevelString, "fatal"))
        {
            specifiedLogLevel = LogLevel::Fatal;
        }
        else if (equalStrings(logLevelString, "error"))
        {
            specifiedLogLevel = LogLevel::Error;
        }
        else if (equalStrings(logLevelString, "warn"))
        {
            specifiedLogLevel = LogLevel::Warn;
        }
        else if (equalStrings(logLevelString, "info"))
        {
            specifiedLogLevel = LogLevel::Info;
        }
        else if (equalStrings(logLevelString, "debug"))
        {
            specifiedLogLevel = LogLevel::Debug;
        }
        else if (equalStrings(logLevelString, "trace"))
        {
            specifiedLogLevel = LogLevel::Trace;
        }
        else
        {
            puts("Invalid value for 'IOX_LOG_LEVEL' environment variable!'");
            puts("Found:");
            puts(logLevelString);
            puts("Allowed is one of: off, fatal, error, warn, info, debug, trace");
        }
    }
    return specifiedLogLevel;
}

} // namespace log
} // namespace iox
