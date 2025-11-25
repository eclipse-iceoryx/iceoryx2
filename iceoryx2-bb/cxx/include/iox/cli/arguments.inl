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

#ifndef IOX_HOOFS_CLI_CLI_DEFINITION_INL
#define IOX_HOOFS_CLI_CLI_DEFINITION_INL

#include "iox/cli/arguments.hpp"

namespace iox
{
namespace cli
{
template <typename T>
inline expected<T, Arguments::Error> Arguments::convertFromString(const Argument_t& stringValue) const noexcept
{
    auto result = convert::from_string<T>(stringValue.c_str());
    if (!result.has_value())
    {
        std::cout << "\"" << stringValue.c_str() << "\" could not be converted to the requested type" << std::endl;
        return err(Error::UNABLE_TO_CONVERT_VALUE);
    }
    return ok(result.value());
}

template <>
inline expected<bool, Arguments::Error> Arguments::convertFromString(const Argument_t& stringValue) const noexcept
{
    if (stringValue != "true" && stringValue != "false")
    {
        std::cout << "\"" << stringValue.c_str() << "\" could not be converted to the requested type" << std::endl;
        return err(Error::UNABLE_TO_CONVERT_VALUE);
    }

    return ok(stringValue == "true");
}

template <typename T>
inline expected<T, Arguments::Error> Arguments::get(const OptionName_t& optionName) const noexcept
{
    for (const auto& a : m_arguments)
    {
        if (a.hasOptionName(optionName))
        {
            return convertFromString<T>(a.value);
        }
    }

    return err(Error::NO_SUCH_VALUE);
}
} // namespace cli
} // namespace iox

#endif // IOX_HOOFS_CLI_CLI_DEFINITION_INL
