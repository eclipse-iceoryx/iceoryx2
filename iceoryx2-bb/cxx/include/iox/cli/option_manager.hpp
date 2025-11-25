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

#ifndef IOX_HOOFS_CLI_OPTION_MANAGER_HPP
#define IOX_HOOFS_CLI_OPTION_MANAGER_HPP

#include "iox/cli/command_line_parser.hpp"
#include "iox/cli/option_definition.hpp"
#include "iox/function.hpp"
#include "iox/std_string_support.hpp"
#include "iox/vector.hpp"

namespace iox
{
namespace cli
{
using CmdAssignments_t = vector<function<void(Arguments&)>, MAX_NUMBER_OF_ARGUMENTS>;

/// @brief Manages command line options which were defined via the IOX_CLI_ macros in a
///        user defined struct.
///        This class ensures that the option values are assigned to the member variables
///        of the struct when the constructor of the IOX_CLI_DEFINITION struct is called.
class OptionManager
{
  public:
    /// @brief Create OptionManager
    /// @param[in] programDescription the description of the application
    /// @param[in] onFailureCallback callback which is called when a syntax error occurs, a required option is missing
    /// or the wrong type as argument value is provided
    OptionManager(const OptionDescription_t& programDescription, const function<void()>& onFailureCallback);

    /// @brief Defines a new option
    /// @param[in] referenceToMember an uninitialized piece of memory where later the content is stored when
    ///            populateDefinedOptions is called
    /// @param[in] shortName the short option name
    /// @param[in] name the long option name
    /// @param[in] description the description of the option
    /// @param[in] optionType the type of option
    /// @param[in] defaultArgumentValue the default value of the option
    template <typename T>
    T defineOption(T& referenceToMember, // not a pointer since it must be always valid
                   const char shortName,
                   const OptionName_t& name,
                   const OptionDescription_t& description,
                   const OptionType optionType,
                   T defaultArgumentValue // not const to enable RTVO
    );

    /// @brief populates all defined options
    /// @param[in] binaryName the name of the binary
    /// @param[in] argc the argument count taken from int main(int argc, char** argv)
    /// @param[in] argv the argument array ptr taken from int main(int argc, char** argv)
    /// @param[in] argcOffset the offset from which the arguments should be parsed
    void populateDefinedOptions(const char*& binaryName, int argc, char** argv, const uint64_t argcOffset);

  private:
    CommandLineParser m_parser;
    OptionDefinition m_optionSet;
    CmdAssignments_t m_assignments;

  private:
    static OptionName_t getLookupName(const char shortName, const OptionName_t& name) noexcept;

    template <typename T>
    T extractOptionArgumentValue(const Arguments& arguments,
                                 const char shortName,
                                 const OptionName_t& name,
                                 const OptionType optionType);
};

} // namespace cli
} // namespace iox

#include "iox/cli/option_manager.inl"

#endif // IOX_HOOFS_CLI_OPTION_MANAGER_HPP
