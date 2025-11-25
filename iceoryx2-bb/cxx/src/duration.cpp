// Copyright (c) 2019, 2021 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 -2022 by Apex.AI Inc. All rights reserved.
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

#include "iox/duration.hpp"
#include "iceoryx_platform/platform_correction.hpp"
#include "iox/assertions.hpp"
#include "iox/logging.hpp"

#include "iox/posix_call.hpp"

namespace iox
{
namespace units
{
struct timespec Duration::timespec(const TimeSpecReference reference) const noexcept
{
    using SEC_TYPE = decltype(std::declval<struct timespec>().tv_sec);
    using NSEC_TYPE = decltype(std::declval<struct timespec>().tv_nsec);

    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    if (reference == TimeSpecReference::None)
    {
        static_assert(sizeof(uint64_t) >= sizeof(SEC_TYPE), "casting might alter result");
        if (this->m_seconds > static_cast<uint64_t>(std::numeric_limits<SEC_TYPE>::max()))
        {
            IOX_LOG(Trace, ": Result of conversion would overflow, clamping to max value!");
            return {std::numeric_limits<SEC_TYPE>::max(), NANOSECS_PER_SEC - 1U};
        }

        const auto tv_sec = static_cast<SEC_TYPE>(this->m_seconds);
        const auto tv_nsec = static_cast<NSEC_TYPE>(this->m_nanoseconds);
        return {tv_sec, tv_nsec};
    }

    struct timespec referenceTime
    {
    };

    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    // AXIVION Next Construct AutosarC++19_03-M5.0.3: False positive! CLOCK_REALTIME and CLOCK_MONOTONIC are of type clockid_t
    const iox_clockid_t clockId{(reference == TimeSpecReference::Epoch) ? CLOCK_REALTIME : CLOCK_MONOTONIC};
    IOX_ENFORCE(
        !IOX_POSIX_CALL(iox_clock_gettime)(clockId, &referenceTime).failureReturnValue(-1).evaluate().has_error(),
        "An error which should never happen occured during 'iox_clock_gettime'!");

    const auto targetTime = Duration(referenceTime) + *this;

    static_assert(sizeof(uint64_t) >= sizeof(SEC_TYPE), "casting might alter result");
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    if (targetTime.m_seconds > static_cast<uint64_t>(std::numeric_limits<SEC_TYPE>::max()))
    {
        IOX_LOG(Trace, ": Result of conversion would overflow, clamping to max value!");
        return {std::numeric_limits<SEC_TYPE>::max(), NANOSECS_PER_SEC - 1U};
    }

    const auto tv_sec = static_cast<SEC_TYPE>(targetTime.m_seconds);
    const auto tv_nsec = static_cast<NSEC_TYPE>(targetTime.m_nanoseconds);
    return {tv_sec, tv_nsec};
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1 : This is not used as shift operator but as stream operator and does not require to implement '<<='
std::ostream& operator<<(std::ostream& stream, const units::Duration t)
{
    stream << t.m_seconds << "s " << t.m_nanoseconds << "ns";
    return stream;
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1 : This is not used as shift operator but as stream operator and does not require to implement '<<='
iox::log::LogStream& operator<<(iox::log::LogStream& stream, const Duration t) noexcept
{
    stream << t.m_seconds << "s " << t.m_nanoseconds << "ns";
    return stream;
}

} // namespace units
} // namespace iox
