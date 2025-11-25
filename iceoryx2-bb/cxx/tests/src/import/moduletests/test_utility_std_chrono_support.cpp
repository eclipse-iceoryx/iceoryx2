// Copyright (c) 2019 - 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox/std_chrono_support.hpp"

#include "test.hpp"

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::units;
using namespace iox::units::duration_literals;

constexpr uint64_t NANOSECS_PER_MILLISECOND = Duration::NANOSECS_PER_MILLISEC;
constexpr uint64_t NANOSECS_PER_SECOND = Duration::NANOSECS_PER_SEC;

class StdChrono_test : public Test
{
};

// BEGIN CONSTRUCTION TESTS

TEST(StdChrono_test, ConstructFromChronoMillisecondsZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "40b02547-8a9d-4ae6-90b2-72e76e5143f0");
    constexpr uint64_t EXPECTED_MILLISECONDS{0U};
    Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.toNanoseconds(), Eq(0U));
}

TEST(StdChrono_test, ConstructFromChronoMillisecondsLessThanOneSecond)
{
    ::testing::Test::RecordProperty("TEST_ID", "ccbcd8df-c146-48ea-a9a0-40abaff31e68");
    constexpr uint64_t EXPECTED_MILLISECONDS{44U};
    Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.toNanoseconds(), Eq(EXPECTED_MILLISECONDS * NANOSECS_PER_MILLISECOND));
}

TEST(StdChrono_test, ConstructFromChronoMillisecondsMoreThanOneSecond)
{
    ::testing::Test::RecordProperty("TEST_ID", "04313e7e-2954-4741-ad0e-6f5a9e5aebce");
    constexpr uint64_t EXPECTED_MILLISECONDS{1001};
    Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.toNanoseconds(), Eq(EXPECTED_MILLISECONDS * NANOSECS_PER_MILLISECOND));
}

TEST(StdChrono_test, ConstructFromChronoMillisecondsMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "553d4ab1-ff8a-437e-8be8-8105e62850c6");
    constexpr uint64_t EXPECTED_MILLISECONDS{std::numeric_limits<int64_t>::max()};
    Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.toMilliseconds(), Eq(EXPECTED_MILLISECONDS));
}

TEST(StdChrono_test, ConstructFromNegativeChronoMillisecondsIsZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "181bc67d-1674-44eb-8784-710306efde30");
    Duration sut = into<Duration>(std::chrono::milliseconds(-1));
    EXPECT_THAT(sut.toNanoseconds(), Eq(0U));
}

TEST(StdChrono_test, ConstructFromChronoNanosecondsZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "3044b00f-a765-417b-a16b-da01c16f7ed0");
    constexpr uint64_t EXPECTED_NANOSECONDS{0U};
    Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.toNanoseconds(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono_test, ConstructFromChronoNanosecondsLessThanOneSecond)
{
    ::testing::Test::RecordProperty("TEST_ID", "a81475d5-5732-44f4-91fc-950792808d7a");
    constexpr uint64_t EXPECTED_NANOSECONDS{424242U};
    Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.toNanoseconds(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono_test, ConstructFromChronoNanosecondsMoreThanOneSecond)
{
    ::testing::Test::RecordProperty("TEST_ID", "43c49a79-28a6-4df4-862d-fc3aaa363191");
    constexpr uint64_t EXPECTED_NANOSECONDS{NANOSECS_PER_SECOND + 42U};
    Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.toNanoseconds(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono_test, ConstructFromChronoNanosecondsMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "a2520ffb-ecc9-4bbb-9f7e-0fbe05383efc");
    constexpr uint64_t EXPECTED_NANOSECONDS{std::numeric_limits<int64_t>::max()};
    Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.toNanoseconds(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono_test, ConstructFromNegativeChronoNanosecondsIsZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "11993efd-9f4f-41b0-a07b-84732d7e6f71");
    Duration sut = into<Duration>(std::chrono::nanoseconds(-1));
    EXPECT_THAT(sut.toNanoseconds(), Eq(0U));
}

// END CONSTRUCTION TESTS

} // namespace
