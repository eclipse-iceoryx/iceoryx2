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
#ifndef IOX_HOOFS_TIME_UNITS_DURATION_INL
#define IOX_HOOFS_TIME_UNITS_DURATION_INL

#include "iox/duration.hpp"

namespace iox
{
namespace units
{
// NOLINTJUSTIFICATION @todo iox-#1617 Seconds_t and Nanoseconds_t should use Newtype pattern to solve this issue
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
inline constexpr Duration::Duration(const Seconds_t seconds, const Nanoseconds_t nanoseconds) noexcept
    : m_seconds(seconds)
    , m_nanoseconds(nanoseconds)
{
    if (nanoseconds >= NANOSECS_PER_SEC)
    {
        const Seconds_t additionalSeconds{static_cast<Seconds_t>(nanoseconds)
                                          / static_cast<Seconds_t>(NANOSECS_PER_SEC)};
        if ((std::numeric_limits<Seconds_t>::max() - additionalSeconds) < m_seconds)
        {
            m_seconds = std::numeric_limits<Seconds_t>::max();
            m_nanoseconds = NANOSECS_PER_SEC - 1U;
        }
        else
        {
            m_seconds += additionalSeconds;
            m_nanoseconds = m_nanoseconds % NANOSECS_PER_SEC;
        }
    }
}

inline constexpr Duration Duration::createDuration(const Seconds_t seconds, const Nanoseconds_t nanoseconds) noexcept
{
    return Duration(seconds, nanoseconds);
}

inline constexpr Duration Duration::max() noexcept
{
    return Duration{std::numeric_limits<Seconds_t>::max(), NANOSECS_PER_SEC - 1U};
}

inline constexpr Duration Duration::zero() noexcept
{
    return Duration{0U, 0U};
}

template <typename T>
inline constexpr uint64_t Duration::positiveValueOrClampToZero(const T value) noexcept
{
    static_assert(std::numeric_limits<T>::is_integer, "only integer types are supported");

    // AXIVION Next Construct AutosarC++19_03-A1.4.3 : Value is a templated an arbitrary integer
    // type that is not necessarily unsigned
    if (value < 0)
    {
        return 0U;
    }

    return static_cast<uint64_t>(value);
}

template <typename T>
inline constexpr Duration Duration::fromNanoseconds(const T value) noexcept
{
    const auto clampedValue = positiveValueOrClampToZero(value);
    const auto seconds = static_cast<Duration::Seconds_t>(clampedValue / Duration::NANOSECS_PER_SEC);
    const auto nanoseconds = static_cast<Duration::Nanoseconds_t>(clampedValue % Duration::NANOSECS_PER_SEC);
    return createDuration(seconds, nanoseconds);
}
template <typename T>
inline constexpr Duration Duration::fromMicroseconds(const T value) noexcept
{
    const auto clampedValue = positiveValueOrClampToZero(value);
    const auto seconds = static_cast<Duration::Seconds_t>(clampedValue / Duration::MICROSECS_PER_SEC);
    const auto nanoseconds = static_cast<Duration::Nanoseconds_t>((clampedValue % Duration::MICROSECS_PER_SEC)
                                                                  * Duration::NANOSECS_PER_MICROSEC);
    return createDuration(seconds, nanoseconds);
}
template <typename T>
inline constexpr Duration Duration::fromMilliseconds(const T value) noexcept
{
    const auto clampedValue = positiveValueOrClampToZero(value);
    const auto seconds = static_cast<Duration::Seconds_t>(clampedValue / Duration::MILLISECS_PER_SEC);
    const auto nanoseconds = static_cast<Duration::Nanoseconds_t>((clampedValue % Duration::MILLISECS_PER_SEC)
                                                                  * Duration::NANOSECS_PER_MILLISEC);
    return createDuration(seconds, nanoseconds);
}
template <typename T>
inline constexpr Duration Duration::fromSeconds(const T value) noexcept
{
    const auto clampedValue = positiveValueOrClampToZero(value);
    constexpr Duration::Seconds_t MAX_SECONDS_BEFORE_OVERFLOW{std::numeric_limits<Duration::Seconds_t>::max()};

    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive, platform-dependent
    if (clampedValue > MAX_SECONDS_BEFORE_OVERFLOW)
    {
        return Duration::max();
    }
    return Duration{static_cast<Duration::Seconds_t>(clampedValue), 0U};
}
template <typename T>
inline constexpr Duration Duration::fromMinutes(const T value) noexcept
{
    const auto clampedValue = positiveValueOrClampToZero(value);
    constexpr uint64_t MAX_MINUTES_BEFORE_OVERFLOW{std::numeric_limits<uint64_t>::max() / Duration::SECS_PER_MINUTE};
    if (clampedValue > MAX_MINUTES_BEFORE_OVERFLOW)
    {
        return Duration::max();
    }
    return Duration{static_cast<Duration::Seconds_t>(clampedValue * Duration::SECS_PER_MINUTE), 0U};
}
template <typename T>
inline constexpr Duration Duration::fromHours(const T value) noexcept
{
    const auto clampedValue = positiveValueOrClampToZero(value);
    constexpr uint64_t MAX_HOURS_BEFORE_OVERFLOW{std::numeric_limits<uint64_t>::max() / Duration::SECS_PER_HOUR};
    if (clampedValue > MAX_HOURS_BEFORE_OVERFLOW)
    {
        return Duration::max();
    }
    return Duration{static_cast<Duration::Seconds_t>(clampedValue * Duration::SECS_PER_HOUR), 0U};
}
template <typename T>
inline constexpr Duration Duration::fromDays(const T value) noexcept
{
    const auto clampedValue = positiveValueOrClampToZero(value);
    constexpr uint64_t SECS_PER_DAY{static_cast<uint64_t>(Duration::HOURS_PER_DAY * Duration::SECS_PER_HOUR)};
    constexpr uint64_t MAX_DAYS_BEFORE_OVERFLOW{std::numeric_limits<uint64_t>::max() / SECS_PER_DAY};
    if (clampedValue > MAX_DAYS_BEFORE_OVERFLOW)
    {
        return Duration::max();
    }
    return Duration{static_cast<Duration::Seconds_t>(clampedValue * SECS_PER_DAY), 0U};
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
inline constexpr Duration::Duration(const struct timeval& value) noexcept
    : Duration(static_cast<Seconds_t>(value.tv_sec), static_cast<Nanoseconds_t>(value.tv_usec) * NANOSECS_PER_MICROSEC)
{
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
inline constexpr Duration::Duration(const struct timespec& value) noexcept
    : Duration(static_cast<Seconds_t>(value.tv_sec), static_cast<Nanoseconds_t>(value.tv_nsec))
{
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
inline constexpr Duration::Duration(const struct itimerspec& value) noexcept
    : Duration(value.it_interval)
{
}

inline constexpr uint64_t Duration::toNanoseconds() const noexcept
{
    constexpr Seconds_t MAX_SECONDS_BEFORE_OVERFLOW{std::numeric_limits<uint64_t>::max()
                                                    / static_cast<uint64_t>(NANOSECS_PER_SEC)};
    constexpr Nanoseconds_t MAX_NANOSECONDS_BEFORE_OVERFLOW{
        static_cast<Nanoseconds_t>(std::numeric_limits<uint64_t>::max() % static_cast<uint64_t>(NANOSECS_PER_SEC))};
    constexpr Duration MAX_DURATION_BEFORE_OVERFLOW{
        createDuration(MAX_SECONDS_BEFORE_OVERFLOW, MAX_NANOSECONDS_BEFORE_OVERFLOW)};

    if (*this > MAX_DURATION_BEFORE_OVERFLOW)
    {
        return std::numeric_limits<uint64_t>::max();
    }

    return (m_seconds * NANOSECS_PER_SEC) + m_nanoseconds;
}

inline constexpr uint64_t Duration::toMicroseconds() const noexcept
{
    constexpr Seconds_t MAX_SECONDS_BEFORE_OVERFLOW{std::numeric_limits<uint64_t>::max() / MICROSECS_PER_SEC};
    constexpr Nanoseconds_t MAX_NANOSECONDS_BEFORE_OVERFLOW{
        static_cast<Nanoseconds_t>(std::numeric_limits<uint64_t>::max() % MICROSECS_PER_SEC) * NANOSECS_PER_MICROSEC};
    constexpr Duration MAX_DURATION_BEFORE_OVERFLOW{
        createDuration(MAX_SECONDS_BEFORE_OVERFLOW, MAX_NANOSECONDS_BEFORE_OVERFLOW)};

    if (*this > MAX_DURATION_BEFORE_OVERFLOW)
    {
        return std::numeric_limits<uint64_t>::max();
    }

    return (m_seconds * MICROSECS_PER_SEC)
           + (static_cast<Seconds_t>(m_nanoseconds) / static_cast<Seconds_t>(NANOSECS_PER_MICROSEC));
}

inline constexpr uint64_t Duration::toMilliseconds() const noexcept
{
    constexpr Seconds_t MAX_SECONDS_BEFORE_OVERFLOW{std::numeric_limits<uint64_t>::max() / MILLISECS_PER_SEC};
    constexpr Nanoseconds_t MAX_NANOSECONDS_BEFORE_OVERFLOW{
        static_cast<Nanoseconds_t>(std::numeric_limits<uint64_t>::max() % MILLISECS_PER_SEC) * NANOSECS_PER_MILLISEC};
    constexpr Duration MAX_DURATION_BEFORE_OVERFLOW{
        createDuration(MAX_SECONDS_BEFORE_OVERFLOW, MAX_NANOSECONDS_BEFORE_OVERFLOW)};

    if (*this > MAX_DURATION_BEFORE_OVERFLOW)
    {
        return std::numeric_limits<uint64_t>::max();
    }

    return (m_seconds * MILLISECS_PER_SEC)
           + (static_cast<Seconds_t>(m_nanoseconds) / static_cast<Seconds_t>(NANOSECS_PER_MILLISEC));
}

inline constexpr uint64_t Duration::toSeconds() const noexcept
{
    return m_seconds;
}

inline constexpr uint64_t Duration::toMinutes() const noexcept
{
    return m_seconds / SECS_PER_MINUTE;
}

inline constexpr uint64_t Duration::toHours() const noexcept
{
    return m_seconds / SECS_PER_HOUR;
}

inline constexpr uint64_t Duration::toDays() const noexcept
{
    return m_seconds / static_cast<uint64_t>(HOURS_PER_DAY * SECS_PER_HOUR);
}

inline constexpr struct timeval Duration::timeval() const noexcept
{
    using SEC_TYPE = decltype(timeval::tv_sec);
    using USEC_TYPE = decltype(timeval::tv_usec);
    static_assert(sizeof(Seconds_t) >= sizeof(SEC_TYPE), "casting might alter result");
    if (m_seconds > static_cast<Seconds_t>(std::numeric_limits<SEC_TYPE>::max()))
    {
        return {std::numeric_limits<SEC_TYPE>::max(), MICROSECS_PER_SEC - 1U};
    }
    // AXIVION Next Construct AutosarC++19_03-M5.0.8, AutosarC++19_03-M5.0.9 : Protected by static assertion
    return {static_cast<SEC_TYPE>(m_seconds), static_cast<USEC_TYPE>(m_nanoseconds / NANOSECS_PER_MICROSEC)};
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
inline constexpr Duration Duration::operator+(const Duration& rhs) const noexcept
{
    Seconds_t seconds{m_seconds + rhs.m_seconds};
    Nanoseconds_t nanoseconds{m_nanoseconds + rhs.m_nanoseconds};
    if (nanoseconds >= NANOSECS_PER_SEC)
    {
        ++seconds;
        nanoseconds -= NANOSECS_PER_SEC;
    }

    const auto sum = createDuration(seconds, nanoseconds);
    if (sum < *this)
    {
        return Duration::max();
    }
    return sum;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
inline constexpr Duration& Duration::operator+=(const Duration& rhs) noexcept
{
    *this = *this + rhs;
    return *this;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
inline constexpr Duration Duration::operator-(const Duration& rhs) const noexcept
{
    if (*this < rhs)
    {
        return Duration::zero();
    }
    Seconds_t seconds{m_seconds - rhs.m_seconds};
    // AXIVION Next Construct AutosarC++19_03-M0.1.9, AutosarC++19_03-A0.1.1, FaultDetection-UnusedAssignments : False positive, variable IS used
    Nanoseconds_t nanoseconds{0U};
    if (m_nanoseconds >= rhs.m_nanoseconds)
    {
        nanoseconds = m_nanoseconds - rhs.m_nanoseconds;
    }
    else
    {
        // AXIVION Next Construct AutosarC++19_03-A4.7.1, AutosarC++19_03-M0.3.1, FaultDetection-IntegerOverflow : It is ensured that m_nanoseconds is never larger than NANOSECS_PER_SEC
        nanoseconds = (NANOSECS_PER_SEC - rhs.m_nanoseconds) + m_nanoseconds;
        --seconds;
    }
    return createDuration(seconds, nanoseconds);
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
inline constexpr Duration& Duration::operator-=(const Duration& rhs) noexcept
{
    *this = *this - rhs;
    return *this;
}

template <typename T>
inline constexpr Duration
Duration::multiplyWith(const std::enable_if_t<!std::is_floating_point<T>::value, T>& rhs) const noexcept
{
    if ((rhs <= static_cast<T>(0)) || (*this == Duration::zero()))
    {
        return Duration::zero();
    }

    static_assert(sizeof(T) <= sizeof(Seconds_t),
                  "only integer types with less or equal to size of uint64_t are allowed for multiplication");
    const auto multiplicator = static_cast<Seconds_t>(rhs);

    const Seconds_t maxBeforeOverflow{std::numeric_limits<Seconds_t>::max() / multiplicator};

    // check if the result of the m_seconds multiplication would already overflow
    if (m_seconds > maxBeforeOverflow)
    {
        return Duration::max();
    }
    const auto durationFromSeconds = Duration(m_seconds * multiplicator, 0U);

    // the m_nanoseconds multiplication cannot exceed the limits of a Duration, since m_nanoseconds is always less than
    // a second and m_seconds can hold 64 bits and the multiplicator is at max 64 bits

    // check if the result of the m_nanoseconds multiplication can easily be converted into a Duration
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    if (m_nanoseconds <= maxBeforeOverflow)
    {
        return durationFromSeconds + Duration::fromNanoseconds(m_nanoseconds * multiplicator);
    }

    // when we reach this, the multiplicator must be larger than 2^32, since smaller values multiplied with the
    // m_nanoseconds(uint32_t) would fit into 64 bits;
    // to accurately determine the result, the calculation is split into a multiplication with the lower 32 bits of the
    // multiplicator and another one with the upper 32 bits;

    // this is the easy part with the lower 32 bits
    const uint64_t multiplicatorLow{static_cast<uint32_t>(multiplicator)};
    const Duration durationFromNanosecondsLow{Duration::fromNanoseconds(m_nanoseconds * multiplicatorLow)};

    // this is the complicated part with the upper 32 bits;
    // the m_nanoseconds are multiplied with the upper 32 bits of the multiplicator shifted by 32 bit to the right, thus
    // having again a multiplication of two 32 bit values whose result fits into a 64 bit variable;
    // one bit of the result represents 2^32 nanoseconds;
    // just shifting left by 32 bits would result in an overflow, therefore blocks of full seconds must be extracted of
    // the result;
    // this cannot be done by dividing through NANOSECS_PER_SEC, since that one is base 1_000_000_000 and the result is
    // base 2^32, therefore the least common multiple can be used to get blocks of full seconds represented with the LSB
    // representing 2^32 nanoseconds;
    // this can then safely be converted to seconds as well as nanoseconds without loosing precision

    // least common multiple of 2^32 and NANOSECONDS_PER_SECOND;
    // for the following calculation it is not important to be the least common multiple, any common multiple will do
    constexpr uint64_t LEAST_COMMON_MULTIPLE{8388608000000000};
    constexpr uint64_t NUMBER_OF_BITS_IN_UINT32{32};
    static_assert((LEAST_COMMON_MULTIPLE % (1ULL << NUMBER_OF_BITS_IN_UINT32)) == 0, "invalid multiple");
    static_assert((LEAST_COMMON_MULTIPLE % NANOSECS_PER_SEC) == 0, "invalid multiple");

    constexpr uint64_t ONE_FULL_BLOCK_OF_SECONDS_ONLY{LEAST_COMMON_MULTIPLE >> NUMBER_OF_BITS_IN_UINT32};
    constexpr uint64_t SECONDS_PER_FULL_BLOCK{LEAST_COMMON_MULTIPLE / NANOSECS_PER_SEC};

    const uint64_t multiplicatorHigh{static_cast<uint32_t>(multiplicator >> NUMBER_OF_BITS_IN_UINT32)};
    const uint64_t nanosecondsFromHigh{m_nanoseconds * multiplicatorHigh};
    const uint64_t fullBlocksOfSecondsOnly{nanosecondsFromHigh / ONE_FULL_BLOCK_OF_SECONDS_ONLY};
    const uint64_t remainingBlockWithFullAndFractionalSeconds{nanosecondsFromHigh % ONE_FULL_BLOCK_OF_SECONDS_ONLY};

    // AXIVION Next Construct AutosarC++19_03-A4.7.1, AutosarC++19_03-M0.3.1, FaultDetection-IntegerOverflow : The logic from above prevents overflows
    const auto durationFromNanosecondsHigh =
        Duration{fullBlocksOfSecondsOnly * SECONDS_PER_FULL_BLOCK, 0U}
        + Duration::fromNanoseconds(remainingBlockWithFullAndFractionalSeconds << NUMBER_OF_BITS_IN_UINT32);

    return durationFromSeconds + durationFromNanosecondsLow + durationFromNanosecondsHigh;
}


template <typename From, typename To>
inline constexpr bool Duration::wouldCastFromFloatingPointProbablyOverflow(const From floatingPoint) const noexcept
{
    static_assert(std::is_floating_point<From>::value, "only floating point is allowed");

    // depending on the internal representation this could be either the last value to not cause an overflow
    // or the first one which causes an overflow;
    // to be safe, this is handled like causing an overflow which would result in undefined behavior when casting to
    // Seconds_t
    constexpr From SECONDS_BEFORE_LIKELY_OVERFLOW{static_cast<From>(std::numeric_limits<To>::max())};
    return floatingPoint >= SECONDS_BEFORE_LIKELY_OVERFLOW;
}

template <typename T>
inline constexpr Duration Duration::fromFloatingPointSeconds(const T floatingPointSeconds) const noexcept
{
    static_assert(std::is_floating_point<T>::value, "only floating point is allowed");

    if (std::isinf(floatingPointSeconds))
    {
        return Duration::max();
    }

    T secondsFull{0};
    T secondsFraction{std::modf(floatingPointSeconds, &secondsFull)};

    if (wouldCastFromFloatingPointProbablyOverflow<T, Seconds_t>(secondsFull))
    {
        return Duration::max();
    }

    return Duration{static_cast<Seconds_t>(secondsFull),
                    static_cast<Nanoseconds_t>(secondsFraction * NANOSECS_PER_SEC)};
}

template <typename T>
inline constexpr Duration
Duration::multiplyWith(const std::enable_if_t<std::is_floating_point<T>::value, T>& rhs) const noexcept
{
    if (std::isnan(rhs))
    {
        return (*this == Duration::zero()) ? Duration::zero() : Duration::max();
    }

    // this must be done after the NAN check in order to prevent to access a signaling NAN
    if ((rhs <= static_cast<T>(0)) || (*this == Duration::zero()))
    {
        return Duration::zero();
    }

    auto durationFromSeconds = fromFloatingPointSeconds<T>(static_cast<T>(m_seconds) * rhs);

    auto resultNanoseconds = static_cast<T>(m_nanoseconds) * rhs;

    if (!wouldCastFromFloatingPointProbablyOverflow<T, uint64_t>(resultNanoseconds))
    {
        return durationFromSeconds + Duration::fromNanoseconds(static_cast<uint64_t>(resultNanoseconds));
    }

    // the multiplication result of nanoseconds would exceed the value an uint64_t can represent
    // -> convert result to seconds and and calculate duration
    auto floatingPointSeconds = resultNanoseconds / NANOSECS_PER_SEC;
    auto durationFromNanoseconds = fromFloatingPointSeconds<T>(floatingPointSeconds);

    return durationFromSeconds + durationFromNanoseconds;
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1 : False positive! Corresponding assignment operator is implemented below
template <typename T>
inline constexpr Duration Duration::operator*(const T& rhs) const noexcept
{
    static_assert(std::is_arithmetic<T>::value, "non arithmetic types are not supported for multiplication");

    return multiplyWith<T>(rhs);
}

template <typename T>
inline constexpr Duration& Duration::operator*=(const T& rhs) noexcept
{
    static_assert(std::is_arithmetic<T>::value, "non arithmetic types are not supported for multiplication");

    *this = multiplyWith<T>(rhs);

    return *this;
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1 : False positive! Corresponding assignment operator is implemented below
// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
template <typename T>
inline constexpr Duration operator*(const T& lhs, const Duration& rhs) noexcept
{
    return rhs * lhs;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
template <typename T>
inline constexpr T& operator*=(T&, const Duration&) noexcept
{
    static_assert(
        always_false_v<T>,
        "Assigning the result of a Duration multiplication with 'operator*=' to an arithmetic type is not supported");
    return T();
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator==(const Duration& lhs, const Duration& rhs) noexcept
{
    return (lhs.m_seconds == rhs.m_seconds) && (lhs.m_nanoseconds == rhs.m_nanoseconds);
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator!=(const Duration& lhs, const Duration& rhs) noexcept
{
    return !(lhs == rhs);
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator<(const Duration& lhs, const Duration& rhs) noexcept
{
    return (lhs.m_seconds < rhs.m_seconds)
           || ((lhs.m_seconds == rhs.m_seconds) && (lhs.m_nanoseconds < rhs.m_nanoseconds));
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator>(const Duration& lhs, const Duration& rhs) noexcept
{
    return (lhs.m_seconds > rhs.m_seconds)
           || ((lhs.m_seconds == rhs.m_seconds) && (lhs.m_nanoseconds > rhs.m_nanoseconds));
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator<=(const Duration& lhs, const Duration& rhs) noexcept
{
    return !(lhs > rhs);
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr bool operator>=(const Duration& lhs, const Duration& rhs) noexcept
{
    return !(lhs < rhs);
}

namespace duration_literals
{
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
inline constexpr Duration operator"" _ns(unsigned long long int value) noexcept
{
    return Duration::fromNanoseconds(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
inline constexpr Duration operator"" _us(unsigned long long int value) noexcept
{
    return Duration::fromMicroseconds(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
inline constexpr Duration operator"" _ms(unsigned long long int value) noexcept
{
    return Duration::fromMilliseconds(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
inline constexpr Duration operator"" _s(unsigned long long int value) noexcept
{
    return Duration::fromSeconds(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
inline constexpr Duration operator"" _m(unsigned long long int value) noexcept
{
    return Duration::fromMinutes(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
inline constexpr Duration operator"" _h(unsigned long long int value) noexcept
{
    return Duration::fromHours(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
inline constexpr Duration operator"" _d(unsigned long long int value) noexcept
{
    return Duration::fromDays(value);
}
} // namespace duration_literals
} // namespace units
} // namespace iox
#endif // IOX_HOOFS_TIME_UNITS_DURATION_INL
