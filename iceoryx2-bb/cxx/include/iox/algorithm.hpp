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
#ifndef IOX_HOOFS_PRIMITIVES_ALGORITHM_HPP
#define IOX_HOOFS_PRIMITIVES_ALGORITHM_HPP

#include "iox/assertions.hpp"
#include "iox/attributes.hpp"
#include "iox/type_traits.hpp"

#include <cstdint>
#include <limits>

namespace iox
{
namespace algorithm
{
/// @brief Returns the maximum gained with operator<() of an arbitrary amount
///          of variables of the same type. Helper function which is required as generic
///          recursive template endpoint.
/// @param T type which implements operator<()
/// @param[in] left value which should be compared
/// @return returns the given argument left
template <typename T>
constexpr T maxVal(const T& left) noexcept;

/// @brief Returns the maximum gained with operator<() of an arbitrary amount
///          of variables of the same type. Helper function which takes two arguments and returns the
///          greater one.
/// @param T type which implements operator<()
/// @param[in] left value which should be compared
/// @param[in] right value which should be compared
/// @return returns the maximum value of the set {left, right}
template <typename T>
constexpr T maxVal(const T& left, const T& right) noexcept;

/// @brief Returns the maximum gained with operator<() of an arbitrary amount
///          of variables of the same type.
/// @param T type which implements operator<()
/// @param[in] left value which should be compared
/// @param[in] right value which should be compared
/// @param[in] args... an arbitrary amount of values
/// @return returns the maximum value of the set {left, right, args...}
template <typename T, typename... Targs>
constexpr T maxVal(const T& left, const T& right, const Targs&... args) noexcept;

/// @brief Returns the minimum gained with operator<() of an arbitrary amount
///          of variables of the same type. Helper function which is required as generic
///          recursive template endpoint.
/// @param T type which implements operator<()
/// @param[in] left value which should be compared
/// @return returns the given argument left
template <typename T>
constexpr T minVal(const T& left) noexcept;

/// @brief Returns the minimum gained with operator<() of an arbitrary amount
///          of variables of the same type. Helper function which takes two arguments and returns the
///          smaller one.
/// @param T type which implements operator<()
/// @param[in] left value which should be compared
/// @param[in] right value which should be compared
/// @return returns the minimum of the set {left, right}
template <typename T>
constexpr T minVal(const T& left, const T& right) noexcept;

/// @brief Returns the minimum gained with operator<() of an arbitrary amount
///          of variables of the same type.
/// @param T type which implements operator<()
/// @param[in] left value which should be compared
/// @param[in] right value which should be compared
/// @param[in] args... an arbitrary amount of values
/// @return returns the minimum of the set {left, right, args...}
template <typename T, typename... Targs>
constexpr T minVal(const T& left, const T& right, const Targs&... args) noexcept;

/// @brief Returns true if T is equal to CompareType, otherwise false
/// @param T type to compare to
/// @param CompareType the type to which T is compared
/// @return true if the types T and CompareType are equal, otherwise false
template <typename T, typename CompareType>
constexpr bool doesContainType() noexcept;

/// @brief Returns true if T is contained the provided type list
/// @param T type to compare to
/// @param CompareType, Next, Remainder the type list in which T should be contained
/// @return true if the T is contained in the type list, otherwise false
template <typename T, typename CompareType, typename Next, typename... Remainder>
constexpr bool doesContainType() noexcept;

/// @brief Finalizes the recursion of doesContainValue
/// @return always false
template <typename T>
inline constexpr bool doesContainValue(const T) noexcept;

/// @brief Returns true if value of T is found in the ValueList, otherwise false
/// @tparam T type of the value to check
/// @tparam ValueList is a list of values to check for a specific value
/// @param[in] value to look for in the ValueList
/// @param[in] firstValueListEntry is the first variadic argument of ValueList
/// @param[in] remainingValueListEntries are the remaining variadic arguments of ValueList
/// @return true if value is contained in the ValueList, otherwise false
/// @note be aware that value is tested for exact equality with the entries of ValueList and regular floating-point
/// comparison rules apply
template <typename T1, typename T2, typename... ValueList>
inline constexpr bool
doesContainValue(const T1 value, const T2 firstValueListEntry, const ValueList... remainingValueListEntries) noexcept;
} // namespace algorithm

namespace internal
{
/// @brief struct to find the best fitting unsigned integer type
template <bool GreaterUint8, bool GreaterUint16, bool GreaterUint32>
struct BestFittingTypeImpl
{
    using Type_t = uint64_t;
};

template <>
struct BestFittingTypeImpl<false, false, false>
{
    using Type_t = uint8_t;
};

template <>
struct BestFittingTypeImpl<true, false, false>
{
    using Type_t = uint16_t;
};

template <>
struct BestFittingTypeImpl<true, true, false>
{
    using Type_t = uint32_t;
};
} // namespace internal
/// @brief get the best fitting unsigned integer type for a given value at compile time
template <uint64_t Value>
struct BestFittingType
{
// gcc warns here that the uint8_t test for BestFittingType<256> is always true... which is correct, but we need it for
// portability anyway
#if defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wtype-limits"
#endif
    using Type_t =
        typename internal::BestFittingTypeImpl<(Value > static_cast<uint64_t>(std::numeric_limits<uint8_t>::max())),
                                               (Value > static_cast<uint64_t>(std::numeric_limits<uint16_t>::max())),
                                               (Value
                                                > static_cast<uint64_t>(std::numeric_limits<uint32_t>::max()))>::Type_t;
#if defined(__GNUC__)
#pragma GCC diagnostic pop
#endif
};

template <uint64_t Value>
using BestFittingType_t = typename BestFittingType<Value>::Type_t;

template <typename T, T Minimum>
struct greater_or_equal
{
  public:
    // AXIVION Next Construct AutosarC++19_03-A12.1.4: this class should behave like a T but which never can be less
    // than Minimum. Adding explicit would defeat the purpose.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    greater_or_equal(T t) noexcept
        : m_value(t)
    {
        if (t < Minimum)
        {
            IOX_LOG(Fatal, "The value '" << t << "' is below '" << Minimum << "'");
            IOX_PANIC("Violating invariant of 'greater_or_equal'");
        }
    }

    // AXIVION Next Construct AutosarC++19_03-A13.5.2,AutosarC++19_03-A13.5.3:this class should behave like a T but
    // which never can be less than Minimum. Adding explicit would defeat the purpose.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    constexpr operator T() const noexcept
    {
        return m_value;
    }

  private:
    T m_value;
};

template <typename T, T Minimum, T Maximum>
struct range
{
  public:
    // AXIVION Next Construct AutosarC++19_03-A12.1.4: this class should behave like a T but with values only in
    // range [Minimum, Maximum] Adding explicit would defeat the purpose.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    range(T t) noexcept
        : m_value(t)
    {
        if (t < Minimum || t > Maximum)
        {
            IOX_LOG(Fatal, "The value '" << t << "' is out of the range [" << Minimum << ", " << Maximum << "]");
            IOX_PANIC("Violating invariant of 'range'");
        }
    }

    // AXIVION Next Construct AutosarC++19_03-A13.5.2, AutosarC++19_03-A13.5.3: this class should behave like a T but
    // with values only in range [Minimum, Maximum]. Adding explicit would defeat the purpose.
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    constexpr operator T() const noexcept
    {
        return m_value;
    }

  private:
    T m_value;
};

/// @brief Checks if an unsigned integer is a power of two
/// @return true if power of two, otherwise false
template <typename T>
constexpr bool isPowerOfTwo(const T n) noexcept
{
    static_assert(std::is_unsigned<T>::value && !std::is_same<T, bool>::value, "Only unsigned integer are allowed!");
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! 'n' can be zero.
    return (n > 0) && ((n & (n - 1U)) == 0U);
}
} // namespace iox

#include "iox/detail/algorithm.inl"

#endif // IOX_HOOFS_PRIMITIVES_ALGORITHM_HPP
