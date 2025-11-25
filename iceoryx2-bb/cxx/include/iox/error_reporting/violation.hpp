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

#ifndef IOX_HOOFS_ERROR_REPORTING_VIOLATION_HPP
#define IOX_HOOFS_ERROR_REPORTING_VIOLATION_HPP

#include <utility>

#include "iox/error_reporting/types.hpp"

namespace iox
{
namespace er
{

// We expect an error to have the following interface
// 1. ErrorCode code() const
// 2. ModuleId module() const

// By default, there are only error codes and violations.
// Custom errors can be added but must satisfy the minimal interface.

// NOLINTNEXTLINE(performance-enum-size) the type is required for error handling
enum class ViolationErrorCode : iox::er::ErrorCode::type
{
    ASSERT_VIOLATION,
    ENFORCE_VIOLATION
};

class Violation
{
  public:
    explicit Violation(ViolationErrorCode code)
        : m_code(static_cast<ErrorCode::type>(code))
    {
    }

    explicit Violation(ErrorCode code)
        : m_code(code)
    {
    }

    Violation(ErrorCode code, ModuleId module)
        : m_code(code)
        , m_module(module)
    {
    }

    ErrorCode code() const
    {
        return m_code;
    }

    ModuleId module() const
    {
        return m_module;
    }

    const char* name() const
    {
        switch (static_cast<ViolationErrorCode>(m_code.value))
        {
        case ViolationErrorCode::ASSERT_VIOLATION:
            return "ASSERT_VIOLATION";
        case ViolationErrorCode::ENFORCE_VIOLATION:
            return "ENFORCE_VIOLATION";
        }
        return "unknown error";
    }

    static const char* moduleName()
    {
        return "ANY";
    }

    bool operator==(const Violation& rhs) const
    {
        return m_code == rhs.m_code && m_module == rhs.m_module;
    }

    bool operator!=(const Violation& rhs) const
    {
        return !(*this == rhs);
    }

    static Violation createAssertViolation()
    {
        return Violation(ViolationErrorCode::ASSERT_VIOLATION);
    }

    static Violation createEnforceViolation()
    {
        return Violation(ViolationErrorCode::ENFORCE_VIOLATION);
    }

  private:
    ErrorCode m_code;
    ModuleId m_module{ModuleId::ANY};
};

} // namespace er

const char* asStringLiteral(const er::ViolationErrorCode error) noexcept;

} // namespace iox

#endif // IOX_HOOFS_ERROR_REPORTING_VIOLATION_HPP
