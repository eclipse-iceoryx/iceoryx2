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

#ifndef IOX_HOOFS_CLI_OPTION_HPP
#define IOX_HOOFS_CLI_OPTION_HPP

#include "iox/cli/types.hpp"

namespace iox
{
namespace cli
{
/// @brief Represents a command line option
struct Option
{
    /// @brief returns true when the name is either equal to the long or
    ///        short option
    /// @param[in] name the option name in question
    bool hasOptionName(const OptionName_t& name) const noexcept;

    /// @brief returns true when the long and short options are equal (does
    ///        not consider value)
    /// @param[in] rhs the other option to which it should be compared
    bool isSameOption(const Option& rhs) const noexcept;

    /// @brief Returns true if rhs is greater than this. It implements a
    ///        lexicographical order.
    /// @param[in] rhs the other option to which it should be compared
    bool operator<(const Option& rhs) const noexcept;

    /// @brief returns true when neither short nor long option is set
    bool isEmpty() const noexcept;

    /// @brief returns true when the long options starts with dash
    bool longOptionNameDoesStartWithDash() const noexcept;

    /// @brief returns true when the short option is a dash
    bool shortOptionNameIsEqualDash() const noexcept;

    /// @brief returns true when the long option name is equal to value
    /// @param[in] value the option name in question
    bool hasLongOptionName(const OptionName_t& value) const noexcept;

    /// @brief returns true when the short option name is equal to value
    /// @param[in] value the option name in question
    bool hasShortOptionName(const char value) const noexcept;

    /// @brief returns true when it contains a short option which is set
    bool hasShortOption() const noexcept;

    /// @brief returns true when it contains a long option which is set
    bool hasLongOption() const noexcept;

    char shortOption = NO_SHORT_OPTION;
    bool isSwitch = false;
    OptionName_t longOption;
    Argument_t value;
};

struct OptionWithDetails : public Option // can this be melt together
{
    /// @brief construct a Option class with additional details
    /// @param[in] option the option
    /// @param[in] description the description of the option
    /// @param[in] type the type of the option
    /// @param[in] typeName the type name of the option
    OptionWithDetails(const Option& option,
                      const OptionDescription_t& description,
                      const OptionType type,
                      const TypeName_t& typeName) noexcept;

    /// @brief Returns true if rhs is greater than this. It implements a
    ///        lexicographical order.
    /// @param[in] rhs the other OptionWithDetails to which it should be compared
    bool operator<(const OptionWithDetails& rhs) const noexcept;

    struct
    {
        OptionDescription_t description;
        OptionType type = OptionType::Switch;
        TypeName_t typeName;
    } details;
};
} // namespace cli
} // namespace iox

#endif // IOX_HOOFS_CLI_OPTION_HPP
