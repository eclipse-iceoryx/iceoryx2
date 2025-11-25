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

#include "iox/cli/option.hpp"

namespace iox
{
namespace cli
{
bool Option::isEmpty() const noexcept
{
    return longOption.empty() && shortOption == NO_SHORT_OPTION;
}

bool Option::longOptionNameDoesStartWithDash() const noexcept
{
    return !longOption.empty() && longOption[0] == '-';
}

bool Option::shortOptionNameIsEqualDash() const noexcept
{
    return shortOption == '-';
}

bool Option::hasLongOptionName(const OptionName_t& value) const noexcept
{
    return (!longOption.empty() && longOption == value);
}

bool Option::hasShortOptionName(const char value) const noexcept
{
    return (shortOption != NO_SHORT_OPTION && shortOption == value);
}

bool Option::hasOptionName(const OptionName_t& name) const noexcept
{
    return hasLongOptionName(name) || (name.size() == 1 && hasShortOptionName(name[0]));
}

bool Option::isSameOption(const Option& rhs) const noexcept
{
    return (shortOption == rhs.shortOption && longOption == rhs.longOption);
}

bool Option::hasShortOption() const noexcept
{
    return (shortOption != NO_SHORT_OPTION);
}

bool Option::hasLongOption() const noexcept
{
    return !longOption.empty();
}

bool Option::operator<(const Option& rhs) const noexcept
{
    if (shortOption != NO_SHORT_OPTION && rhs.shortOption != NO_SHORT_OPTION)
    {
        return shortOption < rhs.shortOption;
    }

    if (!longOption.empty() && rhs.shortOption != NO_SHORT_OPTION)
    {
        return longOption.unchecked_at(0) < rhs.shortOption;
    }

    if (shortOption != NO_SHORT_OPTION && !rhs.longOption.empty())
    {
        return shortOption < rhs.longOption.unchecked_at(0);
    }

    return longOption < rhs.longOption;
}

OptionWithDetails::OptionWithDetails(const Option& option,
                                     const OptionDescription_t& description,
                                     const OptionType type,
                                     const TypeName_t& typeName) noexcept
    : Option{option}
    , details{description, type, typeName}
{
}

bool OptionWithDetails::operator<(const OptionWithDetails& rhs) const noexcept
{
    return Option::operator<(rhs);
}
} // namespace cli
} // namespace iox
