// Copyright (c) 2022 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_UTILITY_STD_STRING_SUPPORT_HPP
#define IOX_HOOFS_UTILITY_STD_STRING_SUPPORT_HPP

#include "iox/detail/convert.hpp"
#include "iox/into.hpp"
#include "iox/optional.hpp"
#include "iox/string.hpp"

#include <cstdint>
#include <ostream>
#include <string>

namespace iox
{

template <>
struct is_custom_string<std::string> : public std::true_type
{
};

namespace internal
{
/// @brief struct to get a pointer to the char array of the std::string
template <>
struct GetData<std::string>
{
    static const char* call(const std::string& data) noexcept
    {
        return data.data();
    }
};

/// @brief struct to get size of a std::string
template <>
struct GetSize<std::string>
{
    static uint64_t call(const std::string& data) noexcept
    {
        return data.size();
    }
};
} // namespace internal

template <uint64_t N>
struct FromImpl<string<N>, std::string>
{
    static std::string fromImpl(const string<N>& value) noexcept;
};

template <uint64_t N>
struct FromImpl<std::string, string<N>>
{
    static string<N> fromImpl(const std::string& value) noexcept;
};

template <uint64_t N>
struct FromImpl<std::string, optional<string<N>>>
{
    static optional<string<N>> fromImpl(const std::string& value) noexcept;
};

template <uint64_t N>
struct FromImpl<std::string, lossy<string<N>>>
{
    static string<N> fromImpl(const std::string& value) noexcept;
};

/// @brief outputs the fixed string on stream
///
/// @param [in] stream is the output stream
/// @param [in] str is the fixed string
///
/// @return the stream output of the fixed string
template <uint64_t Capacity>
std::ostream& operator<<(std::ostream& stream, const string<Capacity>& str) noexcept;

/// @brief A specialization function of convert::from_string for std::string
/// @param v the input string in c type
/// @return an iox::optional<Destination> where, if the return value is iox::nullopt, it indicates a failed conversion
/// process
template <>
inline iox::optional<std::string> convert::from_string(const char* v) noexcept
{
    return iox::optional<std::string>(v);
}

} // namespace iox

#include "iox/detail/std_string_support.inl"

#endif // IOX_HOOFS_UTILITY_STD_STRING_SUPPORT_HPP
