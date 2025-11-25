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
#ifndef IOX_HOOFS_CONTAINER_UNINITIALIZED_ARRAY_TYPE_TRAITS_HPP
#define IOX_HOOFS_CONTAINER_UNINITIALIZED_ARRAY_TYPE_TRAITS_HPP

#include <cstdint>

#include "iox/type_traits.hpp"

namespace iox
{
template <typename ElementType, uint64_t Capacity, template <typename, uint64_t> class Buffer>
class UninitializedArray;

/// @brief struct to check whether an argument is a iox::UninitializedArray
/// @tparam T Type which should be checked
template <typename T>
struct is_iox_array : std::false_type
{
};
/// @brief struct to check whether an argument is a iox::UninitializedArray
/// @tparam T Type which should be checked
/// @tparam N Size of the iox::UninitializedArray
/// @tparam Buffer Type of the buffer of the iox::UninitializedArray
template <typename T, uint64_t N, template <typename, uint64_t> class Buffer>
struct is_iox_array<iox::UninitializedArray<T, N, Buffer>> : std::true_type
{
};
/// @brief struct to check whether an argument is not a iox::UninitializedArray
/// @tparam T Type which should be checked
template <typename T>
using is_not_iox_array_t = iox::negation<is_iox_array<std::decay_t<T>>>;

} // namespace iox

#endif // IOX_HOOFS_CONTAINER_UNINITIALIZED_ARRAY_TYPE_TRAITS_HPP
