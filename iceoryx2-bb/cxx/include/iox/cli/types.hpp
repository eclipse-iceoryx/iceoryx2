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

#ifndef IOX_HOOFS_CLI_TYPES_HPP
#define IOX_HOOFS_CLI_TYPES_HPP

#include "iceoryx_platform/platform_settings.hpp"
#include "iox/string.hpp"

#include <cstdint>

namespace iox
{
namespace cli
{
/// @brief defines the type of command line argument option
enum class OptionType : uint8_t
{
    /// @brief option when provided is true
    Switch,
    /// @brief option with value which has to be provided
    Required,
    /// @brief option with value which can be provided
    Optional
};

static constexpr uint64_t MAX_OPTION_NAME_LENGTH = 32;
static constexpr uint64_t MAX_OPTION_ARGUMENT_LENGTH = 128;
static constexpr uint64_t MAX_OPTION_DESCRIPTION_LENGTH = 1024;
static constexpr uint64_t MAX_TYPE_NAME_LENGTH = 16;
static constexpr char NO_SHORT_OPTION = '\0';
static constexpr uint64_t MAX_NUMBER_OF_ARGUMENTS = 16;

using OptionName_t = string<MAX_OPTION_NAME_LENGTH>;
using OptionDescription_t = string<MAX_OPTION_DESCRIPTION_LENGTH>;
using Argument_t = string<MAX_OPTION_ARGUMENT_LENGTH>;
using TypeName_t = string<MAX_TYPE_NAME_LENGTH>;

} // namespace cli
} // namespace iox

#endif // IOX_HOOFS_CLI_TYPES_HPP
