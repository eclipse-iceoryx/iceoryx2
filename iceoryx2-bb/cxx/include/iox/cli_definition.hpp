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

#ifndef IOX_HOOFS_CLI_CLI_DEFINITION_HPP
#define IOX_HOOFS_CLI_CLI_DEFINITION_HPP

#include "iox/cli/option_manager.hpp"


// NOLINTJUSTIFICATION cannot be realized with templates or constexpr functions due to the
// intended usage of building a class
// NOLINTBEGIN(cppcoreguidelines-macro-usage)

#define IOX_INTERNAL_CMD_LINE_VALUE(type, memberName, defaultValue, shortName, longName, description, optionType)      \
  public:                                                                                                              \
    const type& memberName() const noexcept                                                                            \
    {                                                                                                                  \
        return m_##memberName;                                                                                         \
    }                                                                                                                  \
                                                                                                                       \
  private:                                                                                                             \
    type m_##memberName = [this] {                                                                                     \
        return this->m_optionManager->defineOption<type>(                                                              \
            this->m_##memberName, shortName, longName, description, optionType, defaultValue);                         \
    }()

/// @brief Adds an optional value to the command line
/// @param[in] type the type of the optional value
/// @param[in] memberName the name under which the optional value is accessible
/// @param[in] defaultValue the value when it is not set from outside
/// @param[in] shortName a single character for the short option like '-s' for instance
/// @param[in] longName a long option name under which this can be accessed like '--some-name' for instance
/// @param[in] description a description of the optional value
#define IOX_CLI_OPTIONAL(type, memberName, defaultValue, shortName, longName, description)                             \
    IOX_INTERNAL_CMD_LINE_VALUE(                                                                                       \
        type, memberName, defaultValue, shortName, longName, description, iox::cli::OptionType::Optional)

/// @brief Adds a required value to the command line, if it is not provided the program will print the help and
///        terminate
/// @param[in] type the type of the required value
/// @param[in] memberName the name under which the required value is accessible
/// @param[in] shortName a single character for the short option like '-s' for instance
/// @param[in] longName a long option name under which this can be accessed like '--some-name' for instance
/// @param[in] description a description of the required value
#define IOX_CLI_REQUIRED(type, memberName, shortName, longName, description)                                           \
    IOX_INTERNAL_CMD_LINE_VALUE(                                                                                       \
        type, memberName, type(), shortName, longName, description, iox::cli::OptionType::Required)

/// @brief Adds a switch to the command line
/// @param[in] memberName the name under which the switch is accessible
/// @param[in] shortName a single character for the short option like '-s' for instance
/// @param[in] longName a long option name under which this can be accessed like '--some-name' for instance
/// @param[in] description a description of the switch
#define IOX_CLI_SWITCH(memberName, shortName, longName, description)                                                   \
    IOX_INTERNAL_CMD_LINE_VALUE(bool, memberName, false, shortName, longName, description, iox::cli::OptionType::Switch)

/// @brief Helper macro to create a struct with full command line parsing from argc, argv.
/// @param[in] Name the name of the class/struct
/// @param[in] ProgramDescription a description which describes the task of the program
/// @code
/// // With those macros a struct can be generated easily like this:
/// struct CommandLine
/// {
///     IOX_CLI_DEFINITION(CommandLine);
///
///     IOX_CLI_OPTIONAL(string<100>, stringValue, {"default Value"}, 's', "string-value", "some description");
///     IOX_CLI_REQUIRED(string<100>, anotherString, 'a', "another-string", "some description");
///     IOX_CLI_SWITCH(doStuff, 'd', "do-stuff", "do some stuff - some description");
///     IOX_CLI_OPTIONAL(uint64_t, version, 0, 'v', "version", "some description");
/// };
///
/// // This struct parses all command line arguments and stores them. In
/// // the example above the struct provides access to
/// //   .stringValue()
/// //   .anotherString()
/// //   .doStuff()
/// //   .version()
/// // Via the command line parameters
/// //   -s or --string-value
/// //   -a or --another-string
/// //   -d or --do-stuff
/// //   -v or --version
///
/// int main(int argc, char** argv) {
///   auto cmd = CommandLine::parse(argc, argv, "My program description");
///   std::cout << cmd.stringValue() << " " << cmd.anotherString() << std::endl;
/// }
/// @endcode
#define IOX_CLI_DEFINITION(Name)                                                                                       \
  private:                                                                                                             \
    Name(::iox::cli::OptionManager& optionManager, int argc, char** argv, const uint64_t argcOffset = 1U)              \
        : m_optionManager{&optionManager}                                                                              \
    {                                                                                                                  \
        m_optionManager->populateDefinedOptions(m_binaryName, argc, argv, argcOffset);                                 \
    }                                                                                                                  \
                                                                                                                       \
  public:                                                                                                              \
    static Name parse(                                                                                                 \
        int argc,                                                                                                      \
        char** argv,                                                                                                   \
        const iox::cli::OptionDescription_t& programDescription,                                                       \
        const uint64_t argcOffset = 1U,                                                                                \
        const ::iox::function<void()>& onFailureCallback = [] { std::abort(); })                                       \
    {                                                                                                                  \
        ::iox::cli::OptionManager optionManager(programDescription, onFailureCallback);                                \
        return Name(optionManager, argc, argv, argcOffset);                                                            \
    }                                                                                                                  \
                                                                                                                       \
    const char* binaryName() const noexcept                                                                            \
    {                                                                                                                  \
        return m_binaryName;                                                                                           \
    }                                                                                                                  \
                                                                                                                       \
  private:                                                                                                             \
    ::iox::cli::OptionManager* m_optionManager = nullptr;                                                              \
    const char* m_binaryName = nullptr

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif // IOX_HOOFS_CLI_CLI_DEFINITION_HPP
