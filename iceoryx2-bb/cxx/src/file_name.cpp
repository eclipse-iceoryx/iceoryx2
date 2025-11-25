// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox/file_name.hpp"
#include "iox/detail/path_and_file_verifier.hpp"

namespace iox
{
namespace detail
{
bool file_name_does_contain_invalid_characters(const string<platform::IOX_MAX_FILENAME_LENGTH>& value) noexcept
{
    const auto valueSize = value.size();

    for (uint64_t i{0}; i < valueSize; ++i)
    {
        // AXIVION Next Construct AutosarC++19_03-A3.9.1: Not used as an integer but as actual character
        const char c{value.unchecked_at(i)};

        const bool isSmallLetter{detail::ASCII_A <= c && c <= detail::ASCII_Z};
        const bool isCapitalLetter{detail::ASCII_CAPITAL_A <= c && c <= detail::ASCII_CAPITAL_Z};
        const bool isNumber{detail::ASCII_0 <= c && c <= detail::ASCII_9};
        const bool isSpecialCharacter{c == detail::ASCII_DASH || c == detail::ASCII_DOT || c == detail::ASCII_COLON
                                      || c == detail::ASCII_UNDERSCORE};

        if ((!isSmallLetter && !isCapitalLetter) && (!isNumber && !isSpecialCharacter))
        {
            return true;
        }
    }

    return false;
}

bool file_name_does_contain_invalid_content(const string<platform::IOX_MAX_FILENAME_LENGTH>& value) noexcept
{
    return (value.empty() || value == "." || value == "..");
}
} // namespace detail
} // namespace iox
