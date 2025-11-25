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

#include "iox/type_traits.hpp"

namespace iox
{
// NOLINTJUSTIFICATION See definitions in header file.
// NOLINTBEGIN(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
constexpr const char TypeInfo<int8_t>::NAME[];
constexpr const char TypeInfo<int16_t>::NAME[];
constexpr const char TypeInfo<int32_t>::NAME[];
constexpr const char TypeInfo<int64_t>::NAME[];
constexpr const char TypeInfo<uint8_t>::NAME[];
constexpr const char TypeInfo<uint16_t>::NAME[];
constexpr const char TypeInfo<uint32_t>::NAME[];
constexpr const char TypeInfo<uint64_t>::NAME[];
constexpr const char TypeInfo<bool>::NAME[];
constexpr const char TypeInfo<char>::NAME[];
constexpr const char TypeInfo<float>::NAME[];
constexpr const char TypeInfo<double>::NAME[];
constexpr const char TypeInfo<long double>::NAME[];
// NOLINTEND(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
} // namespace iox
