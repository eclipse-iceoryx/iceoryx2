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

#include "iox/cli/option_definition.hpp"
#include "iox/std_string_support.hpp"

namespace iox
{
namespace cli
{
OptionDefinition::OptionDefinition(const OptionDescription_t& programDescription,
                                   const function<void()>& onFailureCallback) noexcept
    : m_programDescription{programDescription}
    , m_onFailureCallback{onFailureCallback}
{
    constexpr bool IS_SWITCH = true;
    std::move(*this).addOption({{'h', IS_SWITCH, {"help"}, {""}}, {"Display help."}, OptionType::Switch, {""}});
}

optional<OptionWithDetails> OptionDefinition::getOption(const OptionName_t& name) const noexcept
{
    for (const auto& r : m_availableOptions)
    {
        if (r.hasOptionName(name))
        {
            return r;
        }
    }
    return nullopt;
}

OptionDefinition& OptionDefinition::addOption(const OptionWithDetails& option) noexcept
{
    if (option.isEmpty())
    {
        std::cout << "Unable to add option with empty short and long option." << std::endl;
        m_onFailureCallback();
        return *this;
    }

    if (option.longOptionNameDoesStartWithDash())
    {
        std::cout << "The first character of a long option cannot start with dash \"-\" but the option \""
                  << option.longOption << "\" starts with dash." << std::endl;
        m_onFailureCallback();
        return *this;
    }

    if (option.shortOptionNameIsEqualDash())
    {
        std::cout << "Dash \"-\" is not a valid character for a short option." << std::endl;
        m_onFailureCallback();
        return *this;
    }

    for (const auto& registeredOption : m_availableOptions)
    {
        bool isLongOrShortOptionRegistered = false;
        if (registeredOption.hasLongOptionName(option.longOption))
        {
            std::cout << "The longOption \"--" << registeredOption.longOption << "\" is already registered for option "
                      << registeredOption << ". Cannot add option \"" << option << "\"." << std::endl;
            isLongOrShortOptionRegistered = true;
        }

        if (registeredOption.hasShortOptionName(option.shortOption))
        {
            std::cout << "The shortOption \"-" << registeredOption.shortOption << "\" is already registered for option "
                      << registeredOption << ". Cannot add option \"" << option << "\"." << std::endl;
            isLongOrShortOptionRegistered = true;
        }

        if (isLongOrShortOptionRegistered)
        {
            m_onFailureCallback();
            return *this;
        }
    }

    m_availableOptions.emplace_back(option);

    return *this;
}

OptionDefinition& OptionDefinition::addSwitch(const char shortOption,
                                              const OptionName_t& longOption,
                                              const OptionDescription_t& description) noexcept
{
    constexpr bool IS_SWITCH = true;
    return addOption({{shortOption, IS_SWITCH, longOption, {""}}, description, OptionType::Switch, {""}});
}

// NOLINTJUSTIFICATION this is not a user facing API but hidden in a macro
// NOLINTNEXTLINE(readability-function-size)
OptionDefinition& OptionDefinition::addOptional(const char shortOption,
                                                const OptionName_t& longOption,
                                                const OptionDescription_t& description,
                                                const TypeName_t& typeName,
                                                const Argument_t& defaultValue) noexcept
{
    constexpr bool IS_NO_SWITCH = false;
    return addOption(
        {{shortOption, IS_NO_SWITCH, longOption, defaultValue}, description, OptionType::Optional, typeName});
}
OptionDefinition& OptionDefinition::addRequired(const char shortOption,
                                                const OptionName_t& longOption,
                                                const OptionDescription_t& description,
                                                const TypeName_t& typeName) noexcept
{
    constexpr bool IS_NO_SWITCH = false;
    return addOption({{shortOption, IS_NO_SWITCH, longOption, {""}}, description, OptionType::Required, typeName});
}

std::ostream& operator<<(std::ostream& stream, const OptionWithDetails& option) noexcept
{
    if (option.hasShortOption())
    {
        stream << "-" << option.shortOption;
    }
    if (option.hasShortOption() && option.hasLongOption())
    {
        stream << ", ";
    }
    if (option.hasLongOption())
    {
        stream << "--" << option.longOption;
    }

    return stream;
}
} // namespace cli
} // namespace iox
