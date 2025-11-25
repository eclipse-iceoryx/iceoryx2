// Copyright (c) 2020 by Robert Bosch GmbH. All rights reserved.
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
#ifndef IOX_HOOFS_VOCABULARY_STRING_INTERNAL_HPP
#define IOX_HOOFS_VOCABULARY_STRING_INTERNAL_HPP

#include "iox/attributes.hpp"
#include "iox/type_traits.hpp"

#include <cstdint>
#include <cstring>

namespace iox
{
// AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1: Basic numerical type of char shall be used
// AXIVION DISABLE STYLE AutosarC++19_03-A18.1.1: C-style arrays are used to acquire size of c
// array safely, strnlen only accesses N elements which is the maximum capacity of the array
// where N is a compile time constant
template <uint64_t>
class string;

namespace internal
{
template <uint64_t N>
// c array not used here, it is a type alias for easier access
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
using charArray = char[N];

/// @brief struct to get capacity of iox::string/char array/char
/// @note capa is a dummy value for any type other than iox::string and char
template <typename T>
struct GetCapa
{
    static constexpr uint64_t capa{0U};
};

template <uint64_t N>
struct GetCapa<string<N>>
{
    static constexpr uint64_t capa{N};
};

template <uint64_t N>
// used to acquire char array capacity safely at compile time
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
struct GetCapa<char[N]>
{
    static constexpr uint64_t capa{N - 1U};
};

template <>
struct GetCapa<char>
{
    static constexpr uint64_t capa{1U};
};

/// @brief generic empty implementation of the struct to get size of a string
template <typename T>
struct GetSize
{
    static_assert(always_false_v<T>, "\n \
        'GetSize' for the specified type is not implemented!\n \
        Please specialize 'iox::internal::GetSize'!\n");
};

/// @brief struct to get size of iox::string
template <uint64_t N>
struct GetSize<string<N>>
{
    static uint64_t call(const string<N>& data) noexcept
    {
        return data.size();
    }
};

/// @brief struct to get size of char array
template <uint64_t N>
// used to acquire size of c array safely, strnlen only accesses N elements which is the maximum capacity of the array
// where N is a compile time constant
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
struct GetSize<char[N]>
{
    // AXIVION Next Construct FaultDetection-TaintAnalysis : False positive! The size of the type is deduced
    static uint64_t call(const charArray<N>& data) noexcept
    {
        return strnlen(&data[0], N);
    }
};

/// @brief struct to get size of a single char
template <>
struct GetSize<char>
{
    static uint64_t call(char) noexcept
    {
        return 1U;
    }
};

/// @brief generic empty implementation of the struct to get the data of a string
template <typename T>
struct GetData
{
    static_assert(always_false_v<T>, "\n \
        'GetData' for the specified type is not implemented!\n \
        Please specialize 'iox::internal::GetData'!\n");
};

/// @brief struct to get a pointer to the char array of the iox::string
template <uint64_t N>
struct GetData<string<N>>
{
    static const char* call(const string<N>& data) noexcept
    {
        return data.c_str();
    }
};

/// @brief struct to get a pointer to the char array of the string literal
template <uint64_t N>
// provides uniform and safe access (in combination with GetCapa and GetSize) to string like constructs like
// iox::string, string literal, char
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
struct GetData<char[N]>
{
    static const char* call(const charArray<N>& data) noexcept
    {
        return &data[0];
    }
};

/// @brief struct to get a pointer to the single char
template <>
struct GetData<char>
{
    static const char* call(const char& data) noexcept
    {
        // AXIVION Next Construct AutosarC++19_03-A7.5.1 : Used for template meta-programming and
        // safe in this context
        return &data;
    }
};

/// @brief struct to get the sum of the capacities of iox::strings/char arrays/chars
template <typename... Targs>
struct SumCapa;

template <>
struct SumCapa<>
{
    static constexpr uint64_t value{0U};
};

template <typename T, typename... Targs>
struct SumCapa<T, Targs...>
{
    static constexpr uint64_t value{GetCapa<T>::capa + SumCapa<Targs...>::value};
};
} // namespace internal
// AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1
// AXIVION ENABLE STYLE AutosarC++19_03-A18.1.1
} // namespace iox
#endif // IOX_HOOFS_VOCABULARY_STRING_INTERNAL_HPP
