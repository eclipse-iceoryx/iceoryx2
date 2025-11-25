// Copyright (c) 2019 - 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2020 - 2022 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_ERROR_HANDLING_ERROR_HANDLING_HPP
#define IOX_HOOFS_ERROR_HANDLING_ERROR_HANDLING_HPP

// Each module (= some unit with its own errors) must provide the following.

// 1. Define the errors of the module -> see below

// 2. Include the custom reporting implementation
#include "iox/error_reporting/custom/error_reporting.hpp"

// 3. Include the error reporting macro API
#include "iox/error_reporting/macros.hpp"

// additional includes
#include "iox/error_reporting/types.hpp"
#include "iox/log/logstream.hpp"

namespace iox
{
// clang-format off

// NOLINTJUSTIFICATION This macro is usee to define an enum and an array with corresponding enum tag names
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX_HOOFS_ERRORS(error) \
    error(DO_NOT_USE_AS_ERROR_THIS_IS_AN_INTERNAL_MARKER) // keep this always at the end of the error list

// clang-format on

// DO NOT TOUCH THE ENUM, you can doodle around with the lines above!!!

// NOLINTNEXTLINE(performance-enum-size) the type is required for error handling
enum class HoofsError : iox::er::ErrorCode::type
{
    IOX_HOOFS_ERRORS(IOX_CREATE_ERROR_ENUM)
};

const char* asStringLiteral(const HoofsError error) noexcept;

inline log::LogStream& operator<<(log::LogStream& stream, HoofsError value) noexcept
{
    stream << asStringLiteral(value);
    return stream;
}

class HoofsErrorType
{
  public:
    explicit HoofsErrorType(HoofsError code)
        : m_code(static_cast<iox::er::ErrorCode::type>(code))
    {
    }

    static constexpr iox::er::ModuleId module()
    {
        return MODULE_ID;
    }

    iox::er::ErrorCode code() const
    {
        return m_code;
    }

    const char* name() const
    {
        return asStringLiteral(static_cast<HoofsError>(m_code.value));
    }

    static const char* moduleName()
    {
        return "iceoryx_hoofs";
    }

    static constexpr iox::er::ModuleId MODULE_ID{iox::er::ModuleId::HOOFS};

  protected:
    iox::er::ErrorCode m_code;
};

namespace er
{

inline HoofsErrorType toError(HoofsError code)
{
    return HoofsErrorType(code);
}

inline ModuleId toModule(HoofsError)
{
    return HoofsErrorType::MODULE_ID;
}

} // namespace er

} // namespace iox

#endif // IOX_HOOFS_ERROR_HANDLING_ERROR_HANDLING_HPP
