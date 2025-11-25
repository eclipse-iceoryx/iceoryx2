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

#ifndef IOX_HOOFS_CLI_COMMAND_PARSER_HPP
#define IOX_HOOFS_CLI_COMMAND_PARSER_HPP

#include "iox/cli/arguments.hpp"
#include "iox/cli/option_definition.hpp"
#include <cstdint>

namespace iox
{
namespace cli
{
/// @brief Factory class for the CommandLineOption. First, one has to register
///        all switches and options before calling parse. This is required for
///        the help page which is generated and printed on failure as well as
///        for consistency and syntax checks.
class CommandLineParser
{
  public:
    static constexpr uint64_t OPTION_OUTPUT_WIDTH = 45;

  private:
    friend class OptionManager;
    friend Arguments parseCommandLineArguments(const OptionDefinition&, int, char**, const uint64_t) noexcept;

    /// @brief Parses the arguments from the command line.
    ///        Calls onFailureCallback in optionSet when the command line arguments contain illegal syntax or required
    ///        values are not provided and prints the help.
    /// @param[in] optionSet the user defined options, based on those options the Arguments object is
    ///            generated
    /// @param[in] argc number of arguments, see int main(int argc, char** argv)
    /// @param[in] argv the string array of arguments, see int main(int argc, char** argv)
    /// @param[in] argcOffset the starting point for the parsing. 1U starts at the first argument.
    Arguments parse(const OptionDefinition& optionSet, int argc, char** argv, const uint64_t argcOffset = 1U) noexcept;

    void printHelpAndExit() const noexcept;

    /// BEGIN only used in parse to improve readability
    ///
    /// Do not use those internal methods outside of parse. They were written
    /// to improve the readability of the code. None of those functions verify
    /// the pre conditions they require, this has to be done by calling them
    /// in the correct order.
    static bool doesFitIntoString(const char* value, const uint64_t maxLength) noexcept;
    bool areAllRequiredValuesPresent() const noexcept;
    bool hasArguments(const uint64_t argc) const noexcept;
    bool doesOptionStartWithDash(const char* option) const noexcept;
    bool hasNonEmptyOptionName(const char* option) const noexcept;
    bool doesNotHaveLongOptionDash(const char* option) const noexcept;
    bool doesNotExceedLongOptionDash(const char* option) const noexcept;
    bool doesOptionNameFitIntoString(const char* option) const noexcept;
    bool isNextArgumentAValue(const uint64_t position) const noexcept;
    bool isOptionSet(const OptionWithDetails& entry) const noexcept;
    bool doesOptionValueFitIntoString(const char* value) const noexcept;
    bool doesOptionHasSucceedingValue(const OptionWithDetails& entry, const uint64_t position) const noexcept;
    bool hasLexicallyValidOption(const char* value) const noexcept;
    /// END only used in parse to improve readability

    void setDefaultValuesToUnsetOptions() noexcept;

  private:
    uint64_t m_argc = 0;
    char** m_argv = nullptr;
    uint64_t m_argcOffset = 0;

    const OptionDefinition* m_optionSet = nullptr;
    Arguments m_optionValue;
};

/// @copydoc CommandLineParser::parse()
Arguments parseCommandLineArguments(const OptionDefinition& optionSet,
                                    int argc,
                                    char** argv,
                                    const uint64_t argcOffset = 1U) noexcept;

} // namespace cli
} // namespace iox

#endif // IOX_HOOFS_CLI_COMMAND_PARSER_HPP
