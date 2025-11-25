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

#include "test_design_functional_interface_common.hpp"
#include "iox/attributes.hpp"

namespace test_design_functional_interface
{
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters) only for testing purposes
GenericValueError::GenericValueError(const value_t value, const error_t error) noexcept
    : m_value{value}
    , m_error{error}
{
}

GenericValueError::operator bool() const noexcept
{
    return m_value != 0;
}

GenericValueError::value_t& GenericValueError::value() & noexcept
{
    return m_value;
}

const GenericValueError::value_t& GenericValueError::value() const& noexcept
{
    return m_value;
}

GenericValueError::value_t&& GenericValueError::value() && noexcept
{
    // we must use std::move otherwise m_value is not converted into a rvalue
    // which would lead to a compile failure
    // NOLINTNEXTLINE(hicpp-move-const-arg, performance-move-const-arg)
    return std::move(m_value);
}

const GenericValueError::value_t&& GenericValueError::value() const&& noexcept
{
    // we must use std::move otherwise m_value is not converted into a rvalue
    // which would lead to a compile failure
    // NOLINTNEXTLINE(hicpp-move-const-arg, performance-move-const-arg)
    return std::move(m_value);
}

GenericValueError::error_t& GenericValueError::error() & noexcept
{
    return m_error;
}

const GenericValueError::error_t& GenericValueError::error() const& noexcept
{
    return m_error;
}

GenericValueError::error_t&& GenericValueError::error() && noexcept
{
    // we must use std::move otherwise m_value is not converted into a rvalue
    // which would lead to a compile failure
    // NOLINTNEXTLINE(hicpp-move-const-arg, performance-move-const-arg)
    return std::move(m_error);
}

const GenericValueError::error_t&& GenericValueError::error() const&& noexcept
{
    // we must use std::move otherwise m_value is not converted into a rvalue
    // which would lead to a compile failure
    // NOLINTNEXTLINE(hicpp-move-const-arg, performance-move-const-arg)
    return std::move(m_error);
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters) only for testing purposes
GenericPlain::GenericPlain(const int value, const int error)
    : m_isValid{value != INVALID_VALUE}
{
    IOX_DISCARD_RESULT(error);
}

GenericPlain::operator bool() const noexcept
{
    return m_isValid;
}


} // namespace test_design_functional_interface
