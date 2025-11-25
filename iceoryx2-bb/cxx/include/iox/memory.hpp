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
#ifndef IOX_HOOFS_MEMORY_MEMORY_HPP
#define IOX_HOOFS_MEMORY_MEMORY_HPP

#include <cassert>
#include <cstdint>
#include <cstdlib>

namespace iox
{

/// @note value + alignment - 1 must not exceed the maximum value for type T
/// @note alignment must be a power of two
template <typename T>
// AXIVION Next Construct AutosarC++19_03-A2.10.5, AutosarC++19_03-M17.0.3 : The function is in the 'iox' namespace which prevents easy misuse
T align(const T value, const T alignment) noexcept
{
    return (value + (alignment - 1)) & (~alignment + 1);
}

/// @brief allocates aligned memory which can only be free'd by alignedFree
/// @param[in] alignment, alignment of the memory
/// @param[in] size, memory size
/// @return void pointer to the aligned memory
void* alignedAlloc(const size_t alignment, const size_t size) noexcept;

/// @brief frees aligned memory allocated with alignedAlloc
/// @param[in] memory, pointer to the aligned memory
void alignedFree(void* const memory) noexcept;

/// template recursion stopper for maximum alignment calculation
template <std::size_t S = 0>
// AXIVION Next Construct AutosarC++19_03-A2.10.5 : The function is in the 'iox' namespace which prevents easy misuse
constexpr std::size_t maxAlignment() noexcept
{
    return S;
}

/// calculate maximum alignment of supplied types
template <typename T, typename... Args>
// AXIVION Next Construct AutosarC++19_03-A2.10.5 : The function is in the 'iox' namespace which prevents easy misuse
constexpr std::size_t maxAlignment() noexcept
{
    const std::size_t remainingMaxAlignment{maxAlignment<Args...>()};
    const std::size_t currentTypeAligment{alignof(T)};
    return (currentTypeAligment > remainingMaxAlignment) ? currentTypeAligment : remainingMaxAlignment;
}

/// template recursion stopper for maximum size calculation
template <std::size_t S = 0>
constexpr std::size_t maxSize() noexcept
{
    return S;
}

/// calculate maximum size of supplied types
template <typename T, typename... Args>
constexpr std::size_t maxSize() noexcept
{
    return (sizeof(T) > maxSize<Args...>()) ? sizeof(T) : maxSize<Args...>();
}
} // namespace iox
#endif // IOX_HOOFS_MEMORY_MEMORY_HPP
