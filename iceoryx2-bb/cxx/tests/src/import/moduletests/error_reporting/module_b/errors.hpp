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

#ifndef IOX_HOOFS_MODULETESTS_ERROR_REPORTING_MODULE_B_ERRORS_HPP
#define IOX_HOOFS_MODULETESTS_ERROR_REPORTING_MODULE_B_ERRORS_HPP

#include "iox/error_reporting/types.hpp"
#include "iox/error_reporting/violation.hpp"

namespace module_b
{
namespace errors
{

using ErrorCode = iox::er::ErrorCode;
using ModuleId = iox::er::ModuleId;

constexpr ModuleId MODULE_ID{13};

// NOLINTNEXTLINE(performance-enum-size) the type is required for error handling
enum class Code : ErrorCode::type
{
    Unknown = 24,
    OutOfMemory = 37,
    OutOfBounds = 12
};

inline const char* asStringLiteral(Code code)
{
    switch (code)
    {
    case Code::Unknown:
        return "Unknown";
    case Code::OutOfMemory:
        return "OutOfMemory";
    case Code::OutOfBounds:
        return "OutOfBounds";
    }
    // unreachable
    return "unknown error";
}

class Error
{
  public:
    constexpr explicit Error(Code code = Code::Unknown)
        : m_code(code)
    {
    }

    static constexpr ModuleId module()
    {
        return MODULE_ID;
    }

    static const char* moduleName()
    {
        return "Module B";
    }

    ErrorCode code() const
    {
        return ErrorCode(static_cast<ErrorCode::type>(m_code));
    }

    const char* name() const
    {
        return asStringLiteral(m_code);
    }

  protected:
    Code m_code;
};

} // namespace errors
} // namespace module_b

namespace iox
{
namespace er
{

// This definition must exist in this namespace for overload resolution.
// Each module must use a unqiue error enum, e.g. by namespace.
inline module_b::errors::Error toError(module_b::errors::Code code)
{
    return module_b::errors::Error(code);
}

// Any error code of this enum has the same module id.
inline ModuleId toModule(module_b::errors::Code)
{
    return module_b::errors::MODULE_ID;
}

// Specialize to provide concrete error names
template <>
inline const char* toModuleName<module_b::errors::Error>(const module_b::errors::Error& error)
{
    return error.moduleName();
}

// Specialize to provide concrete module names
template <>
inline const char* toErrorName<module_b::errors::Error>(const module_b::errors::Error& error)
{
    return error.name();
}

} // namespace er
} // namespace iox

#endif
