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

#ifndef IOX_HOOFS_REPORTING_ERROR_REPORTING_TYPES_HPP
#define IOX_HOOFS_REPORTING_ERROR_REPORTING_TYPES_HPP

#include <cstdint>
#include <utility>

namespace iox
{
namespace er
{

static constexpr const char* UNKNOWN_ERROR_NAME = "unknown error";
static constexpr const char* UNKNOWN_MODULE_NAME = "unknown module";

// These are lightweight regular read/write types that do not require encapsulation (no invariants
// can be broken).

struct ErrorCode
{
    using type = uint32_t;

    type value;

    constexpr explicit ErrorCode(uint32_t value)
        : value(value)
    {
    }

    bool operator==(const ErrorCode& rhs) const
    {
        return value == rhs.value;
    }

    bool operator!=(const ErrorCode& rhs) const
    {
        return !(*this == rhs);
    }
};

struct ModuleId
{
    using type = uint32_t;

    type value;

    static constexpr type ANY{0};
    static constexpr type HOOFS{1};
    static constexpr type POSH{2};
    static constexpr type BINDING_C{3};

    // User module ids should be larger or equal than this to avoid conflicts
    // with internal modules.
    // All lower values are reserved.
    static constexpr type USER_MODULE_BASE{0x100};

    constexpr explicit ModuleId(uint32_t value = ANY)
        : value(value)
    {
    }

    bool operator==(const ModuleId& rhs) const
    {
        return value == rhs.value;
    }

    bool operator!=(const ModuleId& rhs) const
    {
        return !(*this == rhs);
    }
};

// primary template is the identity
// this can be overriden by modules to handle specific errors
template <typename ErrorLike>
auto toError(ErrorLike&& value)
{
    return std::forward<ErrorLike>(value);
}

template <class Error>
inline ErrorCode toCode(const Error& error)
{
    return error.code();
}

template <>
inline ErrorCode toCode<ErrorCode>(const ErrorCode& error)
{
    return error;
}

template <class Error>
inline ModuleId toModule(const Error& error)
{
    return error.module();
}

template <class Error>
inline const char* toModuleName(const Error& error)
{
    return toError(error).moduleName();
}

template <class Error>
inline const char* toErrorName(const Error& error)
{
    return toError(error).name();
}

} // namespace er
} // namespace iox

// NOLINTJUSTIFICATION These macros are used to create enums and corresponding string arrays with the names of the enum tags
// NOLINTBEGIN(cppcoreguidelines-macro-usage)
#define IOX_CREATE_ERROR_ENUM(name) name,
#define IOX_CREATE_ERROR_STRING(name) #name,
// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX_HOOFS_REPORTING_ERROR_REPORTING_TYPES_HPP
