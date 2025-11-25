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

#ifndef IOX_HOOFS_PRIMITIVES_ICEORYX_HOOFS_TYPES_INL
#define IOX_HOOFS_PRIMITIVES_ICEORYX_HOOFS_TYPES_INL

#include "iox/iceoryx_hoofs_types.hpp"

/// @note since this file will be included by many other files, it should not include other header except
/// iceoryx_platform or STL header

namespace iox
{
// AXIVION Next Construct AutosarC++19_03-M2.10.1 : See declaration in header
namespace log
{
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See declaration in header
inline constexpr const char* asStringLiteral(const LogLevel value) noexcept
{
    switch (value)
    {
    case LogLevel::Off:
        return "LogLevel::Off";
    case LogLevel::Fatal:
        return "LogLevel::Fatal";
    case LogLevel::Error:
        return "LogLevel::Error";
    case LogLevel::Warn:
        return "LogLevel::Warn";
    case LogLevel::Info:
        return "LogLevel::Info";
    case LogLevel::Debug:
        return "LogLevel::Debug";
    case LogLevel::Trace:
        return "LogLevel::Trace";
    }

    return "[Undefined LogLevel]";
}
} // namespace log
} // namespace iox

#endif // IOX_HOOFS_PRIMITIVES_ICEORYX_HOOFS_TYPES_INL
