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

#ifndef IOX2_BB_CLI_ARGUMENTS_HPP
#define IOX2_BB_CLI_ARGUMENTS_HPP

#include "iox2/legacy/cli/option.hpp"
#include "iox2/legacy/cli/types.hpp"
#include "iox2/legacy/detail/convert.hpp"
#include "iox2/legacy/expected.hpp"
#include "iox2/legacy/vector.hpp"

namespace iox2 {
namespace legacy {
namespace cli {
/// @brief This class provides access to the command line argument values.
///        When constructed with the default constructor it is empty. Calling
///        CommandLineParser::parse creates and returns a populated Arguments
///        object.
///        This class should never be used directly. Use the CommandLine builder
///        from 'iox/cli_definition.hpp' to create a struct which contains
///        the values.
class Arguments {
  public:
    enum class Error : uint8_t {
        UNABLE_TO_CONVERT_VALUE,
        NO_SUCH_VALUE
    };

    /// @brief returns the value of a specified option
    /// @tparam T the type of the value
    /// @param[in] optionName either one letter for the shortOption or the whole longOption
    /// @return the contained value if the value is present and convertable, otherwise an Error which describes the
    /// error
    template <typename T>
    expected<T, Error> get(const OptionName_t& optionName) const noexcept;

    /// @brief returns true if the specified switch was set, otherwise false
    /// @param[in] switchName either one letter for the shortOption or the whole longOption
    bool isSwitchSet(const OptionName_t& switchName) const noexcept;

    /// @brief returns the full path name of the binary
    const char* binaryName() const noexcept;

  private:
    template <typename T>
    expected<T, Error> convertFromString(const Argument_t& value) const noexcept;
    friend class CommandLineParser;


  private:
    const char* m_binaryName;
    vector<Option, MAX_NUMBER_OF_ARGUMENTS> m_arguments;
};
} // namespace cli
} // namespace legacy
} // namespace iox2

#include "iox2/legacy/cli/arguments.inl"

#endif // IOX2_BB_CLI_ARGUMENTS_HPP
