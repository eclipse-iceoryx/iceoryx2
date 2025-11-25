// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX_HOOFS_TESTING_LIFETIME_AND_ASSIGNMENT_TRACKER_HPP
#define IOX_HOOFS_TESTING_LIFETIME_AND_ASSIGNMENT_TRACKER_HPP

#include <cstdint>
#include <vector>

namespace iox
{
namespace testing
{
template <typename T = uint64_t, T DEFAULT_VALUE = 0>
class LifetimeAndAssignmentTracker
{
  public:
    LifetimeAndAssignmentTracker()
    {
        stats.cTor++;
        stats.classValue = value;
    }

    // NOLINTNEXTLINE(hicpp-explicit-conversions) we want to use this class in tests transparently to a 'T'
    LifetimeAndAssignmentTracker(const T value)
        : value(value)
    {
        stats.customCTor++;
        stats.classValue = value;
    }

    LifetimeAndAssignmentTracker(const LifetimeAndAssignmentTracker& rhs)
        : value(rhs.value)
    {
        stats.copyCTor++;
        stats.classValue = value;
    }

    LifetimeAndAssignmentTracker(LifetimeAndAssignmentTracker&& rhs) noexcept
        : value(rhs.value)
    {
        stats.moveCTor++;
        stats.classValue = value;
    }

    LifetimeAndAssignmentTracker& operator=(const LifetimeAndAssignmentTracker& rhs)
    {
        if (this != &rhs)
        {
            stats.copyAssignment++;
            value = rhs.value;
            stats.classValue = value;
        }
        return *this;
    }

    LifetimeAndAssignmentTracker& operator=(LifetimeAndAssignmentTracker&& rhs) noexcept
    {
        if (this != &rhs)
        {
            stats.moveAssignment++;
            value = rhs.value;
            stats.classValue = value;
        }
        return *this;
    }

    bool operator==(const LifetimeAndAssignmentTracker& rhs) const
    {
        return value == rhs.value;
    }

    ~LifetimeAndAssignmentTracker()
    {
        stats.dTor++;
        stats.classValue = value;
        stats.dTorOrder.emplace_back(value);
    }

    T& ref()
    {
        return value;
    }

    const T& ref() const
    {
        return value;
    }

    struct Statistics
    {
        uint64_t cTor{0};
        uint64_t customCTor{0};
        uint64_t copyCTor{0};
        uint64_t moveCTor{0};
        uint64_t moveAssignment{0};
        uint64_t copyAssignment{0};
        uint64_t dTor{0};
        T classValue{0};

        std::vector<T> dTorOrder;

        void reset()
        {
            cTor = 0;
            customCTor = 0;
            copyCTor = 0;
            moveCTor = 0;
            moveAssignment = 0;
            copyAssignment = 0;
            dTor = 0;
            classValue = 0;
            dTorOrder.clear();
        }
    };

    //NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables) only used for tests
    static Statistics stats;

    T value = DEFAULT_VALUE;
};

template <typename T = uint64_t, T DEFAULT_VALUE = 0>
class MoveOnlyLifetimeAndAssignmentTracker
{
  public:
    MoveOnlyLifetimeAndAssignmentTracker()
    {
        move_only_stats.cTor++;
        move_only_stats.classValue = value;
    }

    // NOLINTNEXTLINE(hicpp-explicit-conversions) we want to use this class in tests transparently to a 'T'
    MoveOnlyLifetimeAndAssignmentTracker(const T value)
        : value(value)
    {
        move_only_stats.customCTor++;
        move_only_stats.classValue = value;
    }

    MoveOnlyLifetimeAndAssignmentTracker(const MoveOnlyLifetimeAndAssignmentTracker& rhs) = delete;

    MoveOnlyLifetimeAndAssignmentTracker(MoveOnlyLifetimeAndAssignmentTracker&& rhs) noexcept
        : value(rhs.value)
    {
        move_only_stats.moveCTor++;
        move_only_stats.classValue = value;
    }

    MoveOnlyLifetimeAndAssignmentTracker& operator=(const MoveOnlyLifetimeAndAssignmentTracker& rhs) = delete;

    MoveOnlyLifetimeAndAssignmentTracker& operator=(MoveOnlyLifetimeAndAssignmentTracker&& rhs) noexcept
    {
        if (this != &rhs)
        {
            move_only_stats.moveAssignment++;
            value = rhs.value;
            move_only_stats.classValue = value;
        }
        return *this;
    }

    bool operator==(const MoveOnlyLifetimeAndAssignmentTracker& rhs) const
    {
        return value == rhs.value;
    }

    ~MoveOnlyLifetimeAndAssignmentTracker()
    {
        move_only_stats.dTor++;
        move_only_stats.classValue = value;
        move_only_stats.dTorOrder.emplace_back(value);
    }

    T& ref()
    {
        return value;
    }

    const T& ref() const
    {
        return value;
    }

    struct Statistics
    {
        uint64_t cTor{0};
        uint64_t customCTor{0};
        uint64_t copyCTor{0};
        uint64_t moveCTor{0};
        uint64_t moveAssignment{0};
        uint64_t copyAssignment{0};
        uint64_t dTor{0};
        T classValue{0};

        std::vector<T> dTorOrder;

        void reset()
        {
            cTor = 0;
            customCTor = 0;
            copyCTor = 0;
            moveCTor = 0;
            moveAssignment = 0;
            copyAssignment = 0;
            dTor = 0;
            classValue = 0;
            dTorOrder.clear();
        }
    };

    //NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables) only used for tests
    static Statistics move_only_stats;

    T value = DEFAULT_VALUE;
};

//NOLINTBEGIN(cppcoreguidelines-avoid-non-const-global-variables) only used for tests
template <typename T, T DEFAULT_VALUE>
typename LifetimeAndAssignmentTracker<T, DEFAULT_VALUE>::Statistics
    LifetimeAndAssignmentTracker<T, DEFAULT_VALUE>::stats{};

template <typename T, T DEFAULT_VALUE>
typename MoveOnlyLifetimeAndAssignmentTracker<T, DEFAULT_VALUE>::Statistics
    MoveOnlyLifetimeAndAssignmentTracker<T, DEFAULT_VALUE>::move_only_stats{};
//NOLINTEND(cppcoreguidelines-avoid-non-const-global-variables)

} // namespace testing
} // namespace iox

#endif // IOX_HOOFS_TESTING_LIFETIME_AND_ASSIGNMENT_TRACKER_HPP
