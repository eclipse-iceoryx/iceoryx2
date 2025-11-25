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

#include "iox/cli/option_manager.hpp"

namespace iox
{
namespace cli
{
OptionManager::OptionManager(const OptionDescription_t& programDescription, const function<void()>& onFailureCallback)
    : m_optionSet{programDescription, onFailureCallback}
{
}

void OptionManager::populateDefinedOptions(const char*& binaryName, int argc, char** argv, const uint64_t argcOffset)
{
    auto options = m_parser.parse(m_optionSet, argc, argv, argcOffset);

    for (const auto& assignment : m_assignments)
    {
        assignment(options);
    }

    binaryName = options.binaryName();
}

OptionName_t OptionManager::getLookupName(const char shortName, const OptionName_t& name) noexcept
{
    if (shortName == NO_SHORT_OPTION)
    {
        return OptionName_t{TruncateToCapacity, &shortName, 1};
    }

    return name;
}

} // namespace cli
} // namespace iox
