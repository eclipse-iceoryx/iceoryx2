// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
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


#include "iox/detail/hoofs_error_reporting.hpp"

namespace iox
{
// NOLINTJUSTIFICATION Use to map enum tag names to strings
// NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays)
const char* const HOOFS_ERROR_NAMES[] = {IOX_HOOFS_ERRORS(IOX_CREATE_ERROR_STRING)};

const char* asStringLiteral(const HoofsError error) noexcept
{
    auto end =
        static_cast<std::underlying_type<HoofsError>::type>(HoofsError::DO_NOT_USE_AS_ERROR_THIS_IS_AN_INTERNAL_MARKER);
    auto index = static_cast<std::underlying_type<HoofsError>::type>(error);
    if (index >= end)
    {
        return "Unknown Error Code!";
    }
    // NOLINTJUSTIFICATION Bounds are checked and access is safe
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
    return HOOFS_ERROR_NAMES[index];
}
} // namespace iox
