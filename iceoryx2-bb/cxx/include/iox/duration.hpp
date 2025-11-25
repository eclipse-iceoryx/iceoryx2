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
#ifndef IOX_HOOFS_TIME_UNITS_DURATION_HPP
#define IOX_HOOFS_TIME_UNITS_DURATION_HPP

#include "iceoryx_platform/time.hpp" // required for QNX
#include "iox/expected.hpp"
#include "iox/log/logstream.hpp"
#include "iox/logging.hpp"
#include "iox/type_traits.hpp"

#include <cmath>

namespace iox
{
namespace units
{
enum class TimeSpecReference : uint8_t
{
    None,
    Epoch,
    Monotonic
};

class Duration;

namespace duration_literals
{
/// @brief Constructs a new Duration object from nanoseconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr Duration operator"" _ns(unsigned long long int value) noexcept;

/// @brief Constructs a new Duration object from microseconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr Duration operator"" _us(unsigned long long int value) noexcept;

/// @brief Constructs a new Duration object from milliseconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr Duration operator"" _ms(unsigned long long int value) noexcept;

/// @brief Constructs a new Duration object from seconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr Duration operator"" _s(unsigned long long int value) noexcept;

/// @brief Constructs a new Duration object from minutes
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr Duration operator"" _m(unsigned long long int value) noexcept;

/// @brief Constructs a new Duration object from hours
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr Duration operator"" _h(unsigned long long int value) noexcept;

/// @brief Constructs a new Duration object from days
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr Duration operator"" _d(unsigned long long int value) noexcept;
} // namespace duration_literals

/// @code
///   #include <iostream>
///   // ...
///   using namespace units;
///   using namespace units::duration_literals;
///   auto someDays = 2 * 7_d + 5_ns;
///   auto someSeconds = 42_s + 500_ms;
///   IOX_LOG(Info, someDays);
///   IOX_LOG(Info, someDays.nanoSeconds<uint64_t>() << " ns");
///   IOX_LOG(Info, someSeconds.milliSeconds<int64_t>() << " ms");
/// @endcode
class Duration
{
  public:
    // BEGIN CREATION FROM STATIC FUNCTIONS

    /// @brief Constructs a new Duration object from nanoseconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as nanoseconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr Duration fromNanoseconds(const T value) noexcept;

    /// @brief Constructs a new Duration object from microseconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as microseconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr Duration fromMicroseconds(const T value) noexcept;

    /// @brief Constructs a new Duration object from milliseconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as milliseconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr Duration fromMilliseconds(const T value) noexcept;

    /// @brief Constructs a new Duration object from seconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as seconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr Duration fromSeconds(const T value) noexcept;

    /// @brief Constructs a new Duration object from minutes
    /// @tparam T is an integer type for the value
    /// @param[in] value as minutes
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr Duration fromMinutes(const T value) noexcept;

    /// @brief Constructs a new Duration object from hours
    /// @tparam T is an integer type for the value
    /// @param[in] value as hours
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr Duration fromHours(const T value) noexcept;

    /// @brief Constructs a new Duration object from days
    /// @tparam T is an integer type for the value
    /// @param[in] value as days
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr Duration fromDays(const T value) noexcept;

    /// @brief Constructs a new Duration object of maximum allowed length. Useful for functions which should have an
    /// "infinite" timeout.
    static constexpr Duration max() noexcept;

    /// @brief Constructs a new Duration object with a duration of zero
    static constexpr Duration zero() noexcept;
    // END CREATION FROM STATIC FUNCTIONS

    // BEGIN CONSTRUCTORS AND ASSIGNMENT

    /// @brief Construct a Duration object from timeval
    /// @param[in] value as timeval
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr explicit Duration(const struct timeval& value) noexcept;

    /// @brief Construct a Duration object from timespec
    /// @param[in] value as timespec
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr explicit Duration(const struct timespec& value) noexcept;

    /// @brief Construct a Duration object from itimerspec
    /// @param[in] value as itimerspec
    /// @note only it_interval from the itimerspec is used
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr explicit Duration(const struct itimerspec& value) noexcept;

    // END CONSTRUCTORS AND ASSIGNMENT

    // BEGIN COMPARISON
    // AXIVION DISABLE STYLE AutosarC++19_03-A8.4.7 : Each argument is larger than two words
    friend constexpr bool operator==(const Duration& lhs, const Duration& rhs) noexcept;
    friend constexpr bool operator!=(const Duration& lhs, const Duration& rhs) noexcept;
    friend constexpr bool operator<(const Duration& lhs, const Duration& rhs) noexcept;
    friend constexpr bool operator<=(const Duration& lhs, const Duration& rhs) noexcept;
    friend constexpr bool operator>(const Duration& lhs, const Duration& rhs) noexcept;
    friend constexpr bool operator>=(const Duration& lhs, const Duration& rhs) noexcept;
    // AXIVION ENABLE STYLE AutosarC++19_03-A8.4.7
    // END COMPARISON

    // BEGIN ARITHMETIC

    /// @brief Creates Duration object by addition. On overflow duration saturates to Duration::max().
    /// @param[in] rhs is the second summand
    /// @return a new Duration object
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr Duration operator+(const Duration& rhs) const noexcept;

    /// @brief Adds a Duration to itself. On overflow duration saturates to Duration::max().
    /// @param[in] rhs is the second summand
    /// @return a reference to itself
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr Duration& operator+=(const Duration& rhs) noexcept;

    /// @brief Creates Duration object by subtraction. On underflow duration saturates to Duration::zero().
    /// @param[in] rhs is the subtrahend
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
    constexpr Duration operator-(const Duration& rhs) const noexcept;

    /// @brief Subtracts a Duration from itself. On underflow duration saturates to Duration::zero().
    /// @param[in] rhs is the subtrahend
    /// @return a reference to itself
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr Duration& operator-=(const Duration& rhs) noexcept;

    /// @brief Creates Duration object by multiplication.
    /// @tparam T is an arithmetic type for the multiplicator
    /// @param[in] rhs is the multiplicator
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    /// @note A duration of 0 will always result in 0, no matter if multiplied with NaN or +Inf
    /// @note There is no explicit division operator! This can be achieved by multiplication with the inverse of the
    /// divisor.
    /// @note Multiplication of a non-zero duration with NaN and +Inf results in a saturated max duration
    template <typename T>
    constexpr Duration operator*(const T& rhs) const noexcept;

    /// @brief Multiplies a Duration with an arithmetic type and assigns the result to itself.
    /// @tparam T is an arithmetic type for the multiplicator
    /// @param[in] rhs is the multiplicator
    /// @return a reference to itself
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    /// @note A duration of 0 will always result in 0, no matter if multiplied with NaN or +Inf
    /// @note There is no explicit division operator! This can be achieved by multiplication with the inverse of the
    /// divisor.
    /// @note Multiplication of a non-zero duration with NaN and +Inf results in a saturated max duration
    template <typename T>
    constexpr Duration& operator*=(const T& rhs) noexcept;

    // END ARITHMETIC

    // BEGIN CONVERSION

    /// @brief returns the duration in nanoseconds
    /// @note If the duration in nanoseconds is larger than an uint64_t can represent, it will be clamped to the
    /// uint64_t max value.
    constexpr uint64_t toNanoseconds() const noexcept;

    /// @brief returns the duration in microseconds
    /// @note If the duration in microseconds is larger than an uint64_t can represent, it will be clamped to the
    /// uint64_t max value.
    /// @note The remaining nanoseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr uint64_t toMicroseconds() const noexcept;

    /// @brief returns the duration in milliseconds
    /// @note If the duration in milliseconds is larger than an uint64_t can represent, it will be clamped to the
    /// uint64_t max value.
    /// @note The remaining microseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr uint64_t toMilliseconds() const noexcept;

    /// @brief returns the duration in seconds
    /// @note The remaining milliseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr uint64_t toSeconds() const noexcept;

    /// @brief returns the duration in minutes
    /// @note The remaining seconds are truncated, similar to the casting behavior of a float to an int.
    constexpr uint64_t toMinutes() const noexcept;

    /// @brief returns the duration in hours
    /// @note The remaining minutes are truncated, similar to the casting behavior of a float to an int.
    constexpr uint64_t toHours() const noexcept;

    /// @brief returns the duration in days
    /// @note The remaining hours are truncated, similar to the casting behavior of a float to an int.
    constexpr uint64_t toDays() const noexcept;

    /// @brief converts duration in a timespec c struct
    struct timespec timespec(const TimeSpecReference reference = TimeSpecReference::None) const noexcept;

    /// @brief converts duration in a timeval c struct
    ///     timeval::tv_sec = seconds since the Epoch (01.01.1970)
    ///     timeval::tv_usec = microseconds
    constexpr struct timeval timeval() const noexcept;

    // END CONVERSION

    // AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
    friend constexpr Duration duration_literals::operator"" _ns(unsigned long long int value) noexcept;
    friend constexpr Duration duration_literals::operator"" _us(unsigned long long int value) noexcept;
    friend constexpr Duration duration_literals::operator"" _ms(unsigned long long int value) noexcept;
    friend constexpr Duration duration_literals::operator"" _s(unsigned long long int value) noexcept;
    friend constexpr Duration duration_literals::operator"" _m(unsigned long long int value) noexcept;
    friend constexpr Duration duration_literals::operator"" _h(unsigned long long int value) noexcept;
    friend constexpr Duration duration_literals::operator"" _d(unsigned long long int value) noexcept;
    // AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

    // AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
    template <typename T>
    friend constexpr Duration operator*(const T& lhs, const Duration& rhs) noexcept;

    friend std::ostream& operator<<(std::ostream& stream, const Duration t);
    friend iox::log::LogStream& operator<<(iox::log::LogStream& stream, const Duration t) noexcept;

    static constexpr uint32_t SECS_PER_MINUTE{60U};
    static constexpr uint32_t SECS_PER_HOUR{3600U};
    static constexpr uint32_t HOURS_PER_DAY{24U};

    static constexpr uint32_t MILLISECS_PER_SEC{1000U};
    static constexpr uint32_t MICROSECS_PER_SEC{MILLISECS_PER_SEC * 1000U};

    static constexpr uint32_t NANOSECS_PER_MICROSEC{1000U};
    static constexpr uint32_t NANOSECS_PER_MILLISEC{NANOSECS_PER_MICROSEC * 1000U};
    static constexpr uint32_t NANOSECS_PER_SEC{NANOSECS_PER_MILLISEC * 1000U};

  protected:
    using Seconds_t = uint64_t;
    using Nanoseconds_t = uint32_t;

    /// @brief Constructs a Duration from seconds and nanoseconds
    /// @param[in] seconds portion of the duration
    /// @param[in] nanoseconds portion of the duration
    /// @note this is protected to be able to use it in unit tests
    constexpr Duration(const Seconds_t seconds, const Nanoseconds_t nanoseconds) noexcept;

    /// @note this is factory method is necessary to build with msvc due to issues calling a protected constexpr ctor
    /// from public methods
    static constexpr Duration createDuration(const Seconds_t seconds, const Nanoseconds_t nanoseconds) noexcept;

  private:
    template <typename T>
    static constexpr uint64_t positiveValueOrClampToZero(const T value) noexcept;

    template <typename T>
    constexpr Duration fromFloatingPointSeconds(const T floatingPointSeconds) const noexcept;
    template <typename From, typename To>
    constexpr bool wouldCastFromFloatingPointProbablyOverflow(const From floatingPoint) const noexcept;

    template <typename T>
    constexpr Duration multiplyWith(const std::enable_if_t<!std::is_floating_point<T>::value, T>& rhs) const noexcept;

    template <typename T>
    constexpr Duration multiplyWith(const std::enable_if_t<std::is_floating_point<T>::value, T>& rhs) const noexcept;

  private:
    Seconds_t m_seconds{0U};
    Nanoseconds_t m_nanoseconds{0U};
};

/// @brief creates Duration object by multiplying object T with a duration. On overflow
///        duration will saturate to Duration::max()
/// @tparam T is an arithmetic type for the multiplicator
/// @param[in] lhs is the multiplicator
/// @param[in] rhs is the multiplicant
/// @return a new Duration object
/// @attention Since negative durations are not allowed, the duration will be clamped to 0
// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
template <typename T>
constexpr Duration operator*(const T& lhs, const Duration& rhs) noexcept;

/// @brief Dummy implementation with a static assert. Assigning the result of a Duration
/// multiplication with 'operator*=' to an arithmetic type is not supported
// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
template <typename T>
constexpr T& operator*=(T& lhs, const Duration& rhs) noexcept;

/// @brief stream operator for the Duration class
std::ostream& operator<<(std::ostream& stream, const Duration t);

/// @brief Equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator==(const Duration& lhs, const Duration& rhs) noexcept;

/// @brief Not equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration not equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator!=(const Duration& lhs, const Duration& rhs) noexcept;

/// @brief Less than operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is less than rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator<(const Duration& lhs, const Duration& rhs) noexcept;

/// @brief Greater than operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is greater than rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator>(const Duration& lhs, const Duration& rhs) noexcept;

/// @brief Less than or equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is less than or equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator<=(const Duration& lhs, const Duration& rhs) noexcept;

/// @brief Greater than or equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is greater than or equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator>=(const Duration& lhs, const Duration& rhs) noexcept;

} // namespace units
} // namespace iox

#include "iox/detail/duration.inl"

#endif // IOX_HOOFS_TIME_UNITS_DURATION_HPP
