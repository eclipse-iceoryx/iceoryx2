// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_PRIMITIVES_ALGORITHM_INL
#define IOX_HOOFS_PRIMITIVES_ALGORITHM_INL

#include "iox/algorithm.hpp"

namespace iox
{
namespace algorithm
{
template <typename T>
inline constexpr T maxVal(const T& left) noexcept
{
    return left;
}

template <typename T>
inline constexpr T maxVal(const T& left, const T& right) noexcept
{
    return (right < left) ? left : right;
}

template <typename T, typename... Targs>
inline constexpr T maxVal(const T& left, const T& right, const Targs&... args) noexcept
{
    return maxVal(maxVal(left, right), args...);
}

template <typename T>
inline constexpr T minVal(const T& left) noexcept
{
    return left;
}

template <typename T>
inline constexpr T minVal(const T& left, const T& right) noexcept
{
    return (left < right) ? left : right;
}

template <typename T, typename... Targs>
inline constexpr T minVal(const T& left, const T& right, const Targs&... args) noexcept
{
    return minVal(minVal(left, right), args...);
}

template <typename T, typename CompareType>
inline constexpr bool doesContainType() noexcept
{
    return std::is_same<T, CompareType>::value;
}

template <typename T, typename CompareType, typename Next, typename... Remainder>
inline constexpr bool doesContainType() noexcept
{
    return doesContainType<T, CompareType>() || doesContainType<T, Next, Remainder...>();
}

template <typename T>
inline constexpr bool doesContainValue(const T) noexcept
{
    return false;
}

template <typename T1, typename T2, typename... ValueList>
inline constexpr bool
doesContainValue(const T1 value, const T2 firstValueListEntry, const ValueList... remainingValueListEntries) noexcept
{
    // AXIVION Next Line AutosarC++19_03-M6.2.2 : intentional check for exact equality
    return (value == firstValueListEntry) ? true : doesContainValue(value, remainingValueListEntries...);
}

} // namespace algorithm
} // namespace iox

#endif // IOX_HOOFS_PRIMITIVES_ALGORITHM_INL
