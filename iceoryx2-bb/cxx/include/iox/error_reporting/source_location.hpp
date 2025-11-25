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

#ifndef IOX_HOOFS_REPORTING_ERROR_REPORTING_LOCATION_HPP
#define IOX_HOOFS_REPORTING_ERROR_REPORTING_LOCATION_HPP

namespace iox
{
namespace er
{
struct SourceLocation
{
    constexpr SourceLocation(const char* file, int line, const char* function)
        : file(file)
        , line(line)
        , function(function)
    {
    }

    const char* file{nullptr};
    int line{0};
    const char* function{nullptr};
};

} // namespace er
} // namespace iox

// NOLINTNEXTLINE(cppcoreguidelines-macro-usage) macro is required for use of location intrinsics (__FILE__ etc.)
#define IOX_CURRENT_SOURCE_LOCATION                                                                                    \
    iox::er::SourceLocation                                                                                            \
    {                                                                                                                  \
        __FILE__, __LINE__, static_cast<const char*>(__FUNCTION__)                                                     \
    } // NOLINT(cppcoreguidelines-pro-bounds-array-to-pointer-decay,hicpp-no-array-decay)
      // needed for source code location, safely wrapped in macro

#endif // IOX_HOOFS_REPORTING_ERROR_REPORTING_LOCATION_HPP
