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

#ifndef IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_CONFIGURATION_HPP
#define IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_CONFIGURATION_HPP

#include "iox/error_reporting/configuration.hpp"

namespace iox
{
namespace er
{

// Specialize to change the checks (and other options if needed) at compile time.
// this can later also be done depending on a #define to select a header
// but we should avoid to have a #define for each option.
template <>
struct ConfigurationParameters<ConfigurationTag>
{
    static constexpr bool CHECK_ASSERT{true}; /// @todo iox-#1032 deactive for release builds
};

} // namespace er
} // namespace iox

#endif // IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_CONFIGURATION_HPP
