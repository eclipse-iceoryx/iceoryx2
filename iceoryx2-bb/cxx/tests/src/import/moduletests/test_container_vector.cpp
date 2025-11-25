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

#include "iceoryx_hoofs/testing/lifetime_and_assignment_tracker.hpp"
#include "iox/detail/hoofs_error_reporting.hpp"
#include "iox/vector.hpp"

#include "iceoryx_hoofs/testing/fatal_failure.hpp"
#include "test.hpp"

#include <vector>

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::testing;

class vector_test : public Test
{
  public:
    using CTorTest = LifetimeAndAssignmentTracker<>;

    void SetUp() override
    {
        stats.reset();
    }

    CTorTest::Statistics& stats = CTorTest::stats;

    static constexpr uint64_t VECTOR_CAPACITY{10};
    vector<uint64_t, VECTOR_CAPACITY> sut;
};

TEST_F(vector_test, NewlyCreatedVectorIsEmpty)
{
    ::testing::Test::RecordProperty("TEST_ID", "8ebb8b11-d044-459e-b9a1-4a3076c8d49c");
    EXPECT_THAT(sut.empty(), Eq(true));
}

TEST_F(vector_test, NewlyCreatedVectorHasSizeZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "f850b288-df04-43b8-b317-bec76c6c4924");
    EXPECT_THAT(sut.size(), Eq(0U));
}

TEST_F(vector_test, Capacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "e0a244d5-6e01-4cbb-9f9a-ac07cad81a5c");
    EXPECT_THAT(sut.capacity(), Eq(10U));
}

TEST_F(vector_test, NewVectorWithElementsCTorWithZeroElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "618cd7f7-42d7-49e0-a504-3894e34a28f8");
    constexpr uint64_t CAPACITY{42U};
    constexpr int DEFAULT_VALUE{13};
    vector<int, CAPACITY> sut(0, DEFAULT_VALUE);
    EXPECT_THAT(sut.empty(), Eq(true));
}

TEST_F(vector_test, NewVectorWithElementsCTorWithSomeElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "65fd89f4-167e-4844-8264-484d2a57c035");
    constexpr uint64_t CAPACITY{42U};
    constexpr uint64_t ELEMENT_COUNT{37U};
    constexpr int DEFAULT_VALUE{13};
    vector<int, CAPACITY> sut(ELEMENT_COUNT, DEFAULT_VALUE);
    EXPECT_THAT(sut.size(), Eq(ELEMENT_COUNT));
    for (const auto& item : sut)
    {
        EXPECT_THAT(item, Eq(DEFAULT_VALUE));
    }
}

TEST_F(vector_test, NewVectorWithElementsCTorWithCapacityElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "9987c1cb-2266-4bad-b91d-0b171dd87f40");
    constexpr uint64_t CAPACITY{42U};
    constexpr int DEFAULT_VALUE{13};
    vector<int, CAPACITY> sut(CAPACITY, DEFAULT_VALUE);
    EXPECT_THAT(sut.size(), Eq(CAPACITY));
    for (const auto& item : sut)
    {
        EXPECT_THAT(item, Eq(DEFAULT_VALUE));
    }
}

TEST_F(vector_test, NewVectorWithElementsCTorWithMoreThanCapacityElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "a22a3329-d4c1-4ecf-a94f-69a990a35658");
    constexpr uint64_t CAPACITY{42U};
    constexpr uint64_t ELEMENT_COUNT{73U};
    constexpr int DEFAULT_VALUE{13};
    vector<int, CAPACITY> sut(ELEMENT_COUNT, DEFAULT_VALUE);
    EXPECT_THAT(sut.size(), Eq(CAPACITY));
    for (const auto& item : sut)
    {
        EXPECT_THAT(item, Eq(DEFAULT_VALUE));
    }
}

TEST_F(vector_test, EmplaceBackSuccessfulWhenSpaceAvailable)
{
    ::testing::Test::RecordProperty("TEST_ID", "98d17e04-0d2b-4575-a1f0-7b3cd918c54d");
    EXPECT_THAT(sut.emplace_back(5U), Eq(true));
}

TEST_F(vector_test, EmplaceBackFailsWhenSpaceNotAvailable)
{
    ::testing::Test::RecordProperty("TEST_ID", "199e6fc7-5bc4-4896-b211-e04fc668ccd0");
    for (uint64_t i = 0U; i < 10U; ++i)
    {
        EXPECT_THAT(sut.emplace_back(5U), Eq(true));
    }
    EXPECT_THAT(sut.emplace_back(5U), Eq(false));
}

TEST_F(vector_test, PushBackSuccessfulWhenSpaceAvailableLValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "42102325-91fa-45aa-a5cb-2bce785d11c1");
    const int a{5};
    EXPECT_THAT(sut.push_back(a), Eq(true));
    ASSERT_THAT(sut.size(), Eq(1U));
    EXPECT_THAT(sut.at(0), Eq(a));
}

TEST_F(vector_test, PushBackFailsWhenSpaceNotAvailableLValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "e47a9f1b-a039-4740-a4b5-ba81886c1330");
    const int a{5};
    for (uint64_t i = 0U; i < 10U; ++i)
    {
        EXPECT_THAT(sut.push_back(a), Eq(true));
    }
    EXPECT_THAT(sut.push_back(a), Eq(false));
}

TEST_F(vector_test, PushBackSuccessfulWhenSpaceAvailableRValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "47988e05-9c67-4b34-bdee-994552df3fa7");
    EXPECT_THAT(sut.push_back(5U), Eq(true));
    ASSERT_THAT(sut.size(), Eq(1U));
    EXPECT_THAT(sut.at(0), Eq(5U));
}

TEST_F(vector_test, PushBackFailsWhenSpaceNotAvailableRValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "07790d09-110d-4cc7-84d9-3ff28e494c8f");
    for (uint64_t i = 0U; i < 10U; ++i)
    {
        EXPECT_THAT(sut.push_back(5U), Eq(true));
    }
    EXPECT_THAT(sut.push_back(5U), Eq(false));
}

TEST_F(vector_test, PopBackOnEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "03ac7f78-680d-49b1-b7e1-7551006b1545");
    sut.pop_back();
    ASSERT_THAT(sut.size(), Eq(0U));
}

TEST_F(vector_test, PopBackNonEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "ef4ecaa7-d467-4c8a-9ab4-4444b87727d0");
    vector<CTorTest, 10U> sut;
    sut.emplace_back(101U);
    ASSERT_THAT(sut.size(), Eq(1U));
    stats.dTor = 0;
    sut.pop_back();
    ASSERT_THAT(sut.size(), Eq(0U));
    ASSERT_THAT(stats.dTor, Eq(1));
}

TEST_F(vector_test, SizeIncreasesWhenElementIsAdded)
{
    ::testing::Test::RecordProperty("TEST_ID", "2f1814ce-dfc8-4dbe-a7c7-ab004e28a7a2");
    sut.emplace_back(5U);
    EXPECT_THAT(sut.size(), Eq(1U));
}

TEST_F(vector_test, SizeEqualsCapacityWheFull)
{
    ::testing::Test::RecordProperty("TEST_ID", "733985c2-ef1d-4772-9c01-4e26e841581d");
    for (uint64_t i = 0U; i < 10U; ++i)
    {
        sut.emplace_back(5U);
    }
    EXPECT_THAT(sut.size(), Eq(sut.capacity()));
}

TEST_F(vector_test, SizeUnchangedWhenEmplaceFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "6ae42f49-ef4d-4b9c-9360-a6e63a8b9357");
    for (uint64_t i = 0U; i < 10U; ++i)
    {
        sut.emplace_back(5U);
    }
    EXPECT_THAT(sut.emplace_back(5U), Eq(false));
    EXPECT_THAT(sut.size(), Eq(sut.capacity()));
}

TEST_F(vector_test, NotEmptyWhenElementWasAdded)
{
    ::testing::Test::RecordProperty("TEST_ID", "651703b8-0828-471e-9333-547dc1b00295");
    sut.emplace_back(5U);
    EXPECT_THAT(sut.empty(), Eq(false));
}

TEST_F(vector_test, EmptyAfterClear)
{
    ::testing::Test::RecordProperty("TEST_ID", "f08cd8f7-2eca-4ef2-b2c3-d642529ecd60");
    sut.emplace_back(5U);
    sut.clear();
    EXPECT_THAT(sut.empty(), Eq(true));
}

TEST_F(vector_test, SizeZeroAfterClear)
{
    ::testing::Test::RecordProperty("TEST_ID", "7fb46858-5e85-488c-8c10-40c44d412c61");
    sut.emplace_back(5U);
    sut.clear();
    EXPECT_THAT(sut.size(), Eq(0U));
}

TEST_F(vector_test, CopyConstructor)
{
    ::testing::Test::RecordProperty("TEST_ID", "afc46f10-e2fe-4c62-beb3-75c28d18d0f9");
    vector<CTorTest, 10U> sut1;
    sut1.emplace_back(101U);
    sut1.emplace_back(102U);

    vector<CTorTest, 10> sut2(sut1);
    EXPECT_THAT(stats.copyCTor, Eq(2U));
    EXPECT_THAT(sut2.at(0).value, Eq(101U));
    EXPECT_THAT(sut2.at(1).value, Eq(102U));
    EXPECT_THAT(sut2.empty(), Eq(false));
    EXPECT_THAT(sut2.size(), Eq(2));
}

TEST_F(vector_test, CopyConstructorWithEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "438c8835-8545-40e4-b544-d66107507e2f");
    vector<CTorTest, 10U> sut1;
    // NOLINTJUSTIFICATION Testing empty copy
    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
    vector<CTorTest, 10> sut2(sut1);
    EXPECT_THAT(stats.copyCTor, Eq(0U));
    EXPECT_THAT(sut2.size(), Eq(0U));
    EXPECT_THAT(sut2.empty(), Eq(true));
}

TEST_F(vector_test, CopyConstructorWithFullVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "b7420f78-f3ca-4a85-8382-fffe49d94bc2");
    vector<CTorTest, 10U> sut1;
    for (uint64_t i = 0U; i < 10U; ++i)
    {
        sut1.emplace_back(i);
    }

    vector<CTorTest, 10> sut2(sut1);
    for (uint64_t i = 0; i < 10; ++i)
    {
        EXPECT_THAT(sut2.at(i).value, Eq(i));
    }

    EXPECT_THAT(stats.copyCTor, Eq(10U));
    EXPECT_THAT(sut2.size(), Eq(10U));
    EXPECT_THAT(sut2.empty(), Eq(false));
}

TEST_F(vector_test, MoveConstructor)
{
    ::testing::Test::RecordProperty("TEST_ID", "c96fdf10-822b-4872-b3d2-d3745a2ccb52");
    vector<CTorTest, 10U> sut1;
    sut1.emplace_back(8101U);
    sut1.emplace_back(8102U);

    vector<CTorTest, 10U> sut2(std::move(sut1));

    EXPECT_THAT(stats.moveCTor, Eq(2U));
    EXPECT_THAT(sut2.at(0).value, Eq(8101U));
    EXPECT_THAT(sut2.at(1).value, Eq(8102U));
    EXPECT_THAT(sut2.empty(), Eq(false));
    EXPECT_THAT(sut2.size(), Eq(2U));
}

TEST_F(vector_test, MoveConstructorWithEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "1d89b1e1-f62f-49c4-bf04-9aba3dbbecab");
    vector<CTorTest, 10U> sut1;

    vector<CTorTest, 10U> sut2(std::move(sut1));

    EXPECT_THAT(stats.moveCTor, Eq(0U));
    EXPECT_THAT(sut2.size(), Eq(0U));
    EXPECT_THAT(sut2.empty(), Eq(true));
}

TEST_F(vector_test, MoveConstructorWithFullVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "7c873c0f-3d88-4edb-95b4-f40b82c03ade");
    vector<CTorTest, 10U> sut1;
    for (uint64_t i = 0U; i < 10U; ++i)
    {
        sut1.emplace_back(i);
    }

    vector<CTorTest, 10U> sut2(std::move(sut1));

    for (uint64_t i = 0; i < 10; ++i)
    {
        EXPECT_THAT(sut2.at(i).value, Eq(i));
    }

    EXPECT_THAT(stats.moveCTor, Eq(10U));
    EXPECT_THAT(sut2.size(), Eq(10U));
    EXPECT_THAT(sut2.empty(), Eq(false));
}

TEST_F(vector_test, DestructorWithEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "7fa2231d-ca50-4ce9-8588-0de0a8511573");
    {
        vector<CTorTest, 10U> sut1;
    }
    EXPECT_THAT(stats.dTor, Eq(0U));
}

TEST_F(vector_test, DestructorSomeElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "ccbc94af-9cfa-49a6-8d69-426794ac6e83");
    {
        vector<CTorTest, 10U> sut1;
        sut1.emplace_back(891U);
        sut1.emplace_back(9191U);
        sut1.emplace_back(1U);
    }
    EXPECT_THAT(stats.dTor, Eq(3U));
}

TEST_F(vector_test, DestructorWithFullVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "c439128d-de50-4af0-bb56-b219d0326afd");
    constexpr uint64_t CAPACITY{10};
    {
        vector<CTorTest, CAPACITY> sut1;
        for (uint64_t i = 0U; i < CAPACITY; ++i)
        {
            sut1.emplace_back(1231U);
        }
    }

    EXPECT_THAT(stats.dTor, Eq(CAPACITY));
}

TEST_F(vector_test, EmplacingElementInTheMiddleCallsDTor)
{
    ::testing::Test::RecordProperty("TEST_ID", "09a217bb-690e-4120-8e06-198e9056e26e");
    constexpr uint64_t CAPACITY_OF_VECTOR{10U};
    constexpr uint64_t EXPECTED_NUMBER_OF_CTOR_CALLS{CAPACITY_OF_VECTOR};
    constexpr uint64_t EMPLACE_POSITION{5U};
    {
        vector<CTorTest, CAPACITY_OF_VECTOR> sut;
        for (uint64_t i = 0U; i < CAPACITY_OF_VECTOR - 1U; ++i)
        {
            sut.emplace_back(1234U);
        }

        EXPECT_THAT(stats.customCTor, Eq(EXPECTED_NUMBER_OF_CTOR_CALLS - 1U));
        EXPECT_TRUE(sut.emplace(EMPLACE_POSITION, 42U));
        EXPECT_THAT(stats.customCTor, Eq(EXPECTED_NUMBER_OF_CTOR_CALLS));
        EXPECT_THAT(stats.moveCTor, Eq(1U));
        EXPECT_THAT(stats.moveAssignment, Eq(CAPACITY_OF_VECTOR - 1U - EMPLACE_POSITION - 1U));
        EXPECT_THAT(stats.dTor, Eq(1U));
    }
    // Last element in the vector is moved and not constructed, hence #moveCTor + #customCTor = #dTor
    EXPECT_THAT(stats.moveCTor, Eq(1U));
    EXPECT_THAT(stats.customCTor, Eq(EXPECTED_NUMBER_OF_CTOR_CALLS));
    EXPECT_THAT(stats.dTor, Eq(EXPECTED_NUMBER_OF_CTOR_CALLS + 1U));
}

TEST_F(vector_test, CopyAssignmentWithEmptySource)
{
    ::testing::Test::RecordProperty("TEST_ID", "3f64706d-b370-41b4-91e1-3e319cd6c14a");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(812U);
    sut1.emplace_back(81122U);
    sut1.emplace_back(8132U);

    sut1 = sut2;

    EXPECT_THAT(stats.dTor, Eq(3U));
    EXPECT_THAT(stats.copyAssignment, Eq(0U));
    EXPECT_THAT(stats.copyCTor, Eq(0U));
    EXPECT_THAT(sut1.size(), Eq(0U));
    EXPECT_THAT(sut1.empty(), Eq(true));
}

TEST_F(vector_test, CopyAssignmentWithEmptyDestination)
{
    ::testing::Test::RecordProperty("TEST_ID", "c3bb0ad8-c099-438c-afc4-d41a22dc4e2f");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(5812U);
    sut1.emplace_back(581122U);
    sut1.emplace_back(58132U);

    sut2 = sut1;

    EXPECT_THAT(stats.dTor, Eq(0U));
    EXPECT_THAT(stats.copyAssignment, Eq(0U));
    EXPECT_THAT(stats.copyCTor, Eq(3U));
    EXPECT_THAT(sut2.size(), Eq(3U));
    EXPECT_THAT(sut2.empty(), Eq(false));

    EXPECT_THAT(sut2.at(0U).value, Eq(5812U));
    EXPECT_THAT(sut2.at(1U).value, Eq(581122U));
    EXPECT_THAT(sut2.at(2U).value, Eq(58132U));
}

TEST_F(vector_test, CopyAssignmentWithLargerDestination)
{
    ::testing::Test::RecordProperty("TEST_ID", "39353120-5606-43b6-8909-a6751a801331");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(5842U);
    sut1.emplace_back(584122U);
    sut1.emplace_back(58432U);
    sut1.emplace_back(58432U);
    sut2.emplace_back(313U);
    sut2.emplace_back(3131U);

    sut1 = sut2;

    EXPECT_THAT(stats.dTor, Eq(2U));
    EXPECT_THAT(stats.copyAssignment, Eq(2U));
    EXPECT_THAT(stats.copyCTor, Eq(0U));
    EXPECT_THAT(sut1.size(), Eq(2U));
    EXPECT_THAT(sut1.empty(), Eq(false));
    EXPECT_THAT(sut1.at(0U).value, Eq(313U));
    EXPECT_THAT(sut1.at(1U).value, Eq(3131U));
}

TEST_F(vector_test, CopyAssignmentWithLargerSource)
{
    ::testing::Test::RecordProperty("TEST_ID", "956303d2-0853-410f-8d44-347a2c5258fe");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(15842U);
    sut1.emplace_back(1584122U);
    sut1.emplace_back(158432U);
    sut1.emplace_back(158432U);
    sut2.emplace_back(1313U);
    sut2.emplace_back(13131U);

    sut2 = sut1;

    EXPECT_THAT(stats.dTor, Eq(0U));
    EXPECT_THAT(stats.copyAssignment, Eq(2U));
    EXPECT_THAT(stats.copyCTor, Eq(2U));
    EXPECT_THAT(sut2.size(), Eq(4U));
    EXPECT_THAT(sut2.empty(), Eq(false));
    EXPECT_THAT(sut2.at(0U).value, Eq(15842U));
    EXPECT_THAT(sut2.at(1U).value, Eq(1584122U));
    EXPECT_THAT(sut2.at(2U).value, Eq(158432U));
    EXPECT_THAT(sut2.at(3U).value, Eq(158432U));
}

TEST_F(vector_test, ReverseDestructionOrderInCopyAssignment)
{
    ::testing::Test::RecordProperty("TEST_ID", "00ba138d-a805-4261-ac54-5eeea605e50c");
    constexpr uint64_t VECTOR_CAPACITY{10};
    vector<CTorTest, VECTOR_CAPACITY> sut1;
    vector<CTorTest, VECTOR_CAPACITY> sut2;
    for (uint64_t i{0}; i < VECTOR_CAPACITY; ++i)
    {
        sut1.emplace_back(i);
    }
    sut1 = sut2;

    EXPECT_THAT(stats.dTor, Eq(VECTOR_CAPACITY));
    ASSERT_THAT(stats.dTorOrder.size(), Eq(VECTOR_CAPACITY));
    for (size_t i{0}; i < VECTOR_CAPACITY; ++i)
    {
        EXPECT_THAT(stats.dTorOrder[i], Eq(VECTOR_CAPACITY - 1 - i));
    }
}

TEST_F(vector_test, ReverseDestructionOrderInMoveAssignment)
{
    ::testing::Test::RecordProperty("TEST_ID", "7a523770-7eab-4405-a9c1-a1b451534eb0");
    constexpr uint64_t VECTOR_CAPACITY{10};
    vector<CTorTest, VECTOR_CAPACITY> sut1;
    vector<CTorTest, VECTOR_CAPACITY> sut2;
    for (uint64_t i{0}; i < VECTOR_CAPACITY; ++i)
    {
        sut1.emplace_back(i + 1);
    }
    sut1 = std::move(sut2);

    EXPECT_THAT(stats.dTor, Eq(VECTOR_CAPACITY));
    ASSERT_THAT(stats.dTorOrder.size(), Eq(VECTOR_CAPACITY));
    for (size_t i{0}; i < VECTOR_CAPACITY; ++i)
    {
        EXPECT_THAT(stats.dTorOrder[i], Eq(VECTOR_CAPACITY - i));
    }
}

TEST_F(vector_test, MoveAssignmentWithEmptySource)
{
    ::testing::Test::RecordProperty("TEST_ID", "dc8c2211-e8f6-4a49-a1bb-8344894c017b");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(812U);
    sut1.emplace_back(81122U);
    sut1.emplace_back(8132U);

    sut1 = std::move(sut2);

    EXPECT_THAT(stats.dTor, Eq(3U));
    EXPECT_THAT(stats.moveAssignment, Eq(0U));
    EXPECT_THAT(stats.moveCTor, Eq(0U));
    EXPECT_THAT(sut1.size(), Eq(0U));
    EXPECT_THAT(sut1.empty(), Eq(true));
}

TEST_F(vector_test, MoveAssignmentWithEmptyDestination)
{
    ::testing::Test::RecordProperty("TEST_ID", "4e9b8dba-4ad3-4281-af5b-e9bb73b8b246");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(5812U);
    sut1.emplace_back(581122U);
    sut1.emplace_back(58132U);

    sut2 = std::move(sut1);

    EXPECT_THAT(stats.dTor, Eq(3U));
    EXPECT_THAT(stats.moveAssignment, Eq(0U));
    EXPECT_THAT(stats.moveCTor, Eq(3U));
    EXPECT_THAT(sut2.size(), Eq(3U));
    EXPECT_THAT(sut2.empty(), Eq(false));
    EXPECT_THAT(sut2.at(0U).value, Eq(5812U));
    EXPECT_THAT(sut2.at(1U).value, Eq(581122U));
    EXPECT_THAT(sut2.at(2U).value, Eq(58132U));
}

TEST_F(vector_test, MoveAssignmentWithLargerDestination)
{
    ::testing::Test::RecordProperty("TEST_ID", "f41e5f13-19bc-4876-a1d0-32c57f06902f");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(5842U);
    sut1.emplace_back(584122U);
    sut1.emplace_back(58432U);
    sut1.emplace_back(58432U);
    sut2.emplace_back(313U);
    sut2.emplace_back(3131U);

    sut1 = std::move(sut2);

    EXPECT_THAT(stats.dTor, Eq(4U));
    EXPECT_THAT(stats.moveAssignment, Eq(2U));
    EXPECT_THAT(stats.moveCTor, Eq(0U));
    EXPECT_THAT(sut1.size(), Eq(2U));
    EXPECT_THAT(sut1.empty(), Eq(false));
    EXPECT_THAT(sut1.at(0U).value, Eq(313U));
    EXPECT_THAT(sut1.at(1U).value, Eq(3131U));
}

TEST_F(vector_test, MoveAssignmentWithLargerSource)
{
    ::testing::Test::RecordProperty("TEST_ID", "b840951e-9d80-49d4-9c8f-3a21b4136ba8");
    vector<CTorTest, 10U> sut1;
    vector<CTorTest, 10U> sut2;
    sut1.emplace_back(15842U);
    sut1.emplace_back(1584122U);
    sut1.emplace_back(158432U);
    sut1.emplace_back(158432U);
    sut2.emplace_back(1313U);
    sut2.emplace_back(13131U);

    sut2 = std::move(sut1);

    EXPECT_THAT(stats.dTor, Eq(4U));
    EXPECT_THAT(stats.moveAssignment, Eq(2U));
    EXPECT_THAT(stats.moveCTor, Eq(2U));
    EXPECT_THAT(sut2.size(), Eq(4U));
    EXPECT_THAT(sut2.empty(), Eq(false));
    EXPECT_THAT(sut2.at(0U).value, Eq(15842U));
    EXPECT_THAT(sut2.at(1U).value, Eq(1584122U));
    EXPECT_THAT(sut2.at(2U).value, Eq(158432U));
    EXPECT_THAT(sut2.at(3U).value, Eq(158432U));
}

TEST_F(vector_test, BeginEndIteratorAreTheSameWhenEmpty)
{
    ::testing::Test::RecordProperty("TEST_ID", "0a9a9760-5f68-436a-8331-ed9956d95b10");
    EXPECT_THAT(sut.begin() == sut.end(), Eq(true));
}

TEST_F(vector_test, BeginEndConstIteratorAreTheSameWhenEmpty)
{
    ::testing::Test::RecordProperty("TEST_ID", "51a9a205-dfff-4abe-b68e-1254d46865f0");
    // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    EXPECT_THAT(const_cast<const decltype(sut)*>(&sut)->begin() == const_cast<const decltype(sut)*>(&sut)->end(),
                Eq(true));
}

TEST_F(vector_test, BeginIteratorComesBeforeEndIteratorWhenNotEmpty)
{
    ::testing::Test::RecordProperty("TEST_ID", "26ab3394-ec0e-4f12-bcac-73d7918bcdbb");
    sut.emplace_back(1U);
    EXPECT_THAT(sut.begin() < sut.end(), Eq(true));
}

TEST_F(vector_test, BeginConstIteratorComesBeforeEndConstIteratorWhenNotEmpty)
{
    ::testing::Test::RecordProperty("TEST_ID", "c1a101ff-c840-45d2-acf8-f2de2fd504c7");
    sut.emplace_back(1U);
    // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    EXPECT_THAT(const_cast<const decltype(sut)*>(&sut)->begin() < const_cast<const decltype(sut)*>(&sut)->end(),
                Eq(true));
}

TEST_F(vector_test, BeginIteratorComesBeforeEndIteratorWhenFull)
{
    ::testing::Test::RecordProperty("TEST_ID", "f20cda46-0941-440e-87cb-a0a111719182");
    for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        sut.emplace_back(i);
    }
    EXPECT_THAT(sut.begin() < sut.end(), Eq(true));
}

TEST_F(vector_test, BeginConstIteratorComesBeforeEndConstIteratorWhenFull)
{
    ::testing::Test::RecordProperty("TEST_ID", "9912c12f-25a4-47f3-a3a6-714c543dd882");
    for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        sut.emplace_back(i);
    }
    // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    EXPECT_THAT(const_cast<const decltype(sut)*>(&sut)->begin() < const_cast<const decltype(sut)*>(&sut)->end(),
                Eq(true));
}

TEST_F(vector_test, IteratorIteratesThroughNonEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "caf1508b-4ea5-4a0d-bba4-b7c0810e236d");
    constexpr uint64_t INITIAL_VALUE = 42U;
    sut.emplace_back(INITIAL_VALUE);
    sut.emplace_back(INITIAL_VALUE + 1U);
    sut.emplace_back(INITIAL_VALUE + 2U);
    const uint64_t EXPECTED_END_INDEX = sut.size();

    uint64_t count = 0U;
    for (auto& v : sut)
    {
        EXPECT_THAT(v, Eq(INITIAL_VALUE + count));
        ++count;
    }
    EXPECT_THAT(count, Eq(EXPECTED_END_INDEX));
}

TEST_F(vector_test, ConstIteratorIteratesThroughNonEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "959fcdac-ca00-4765-a247-947f6fc9e00f");
    constexpr uint64_t INITIAL_VALUE{142U};
    sut.emplace_back(INITIAL_VALUE);
    sut.emplace_back(INITIAL_VALUE + 1U);
    sut.emplace_back(INITIAL_VALUE + 2U);
    const uint64_t EXPECTED_END_COUNT = sut.size();

    uint64_t count = 0U;
    // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    for (const auto& v : *const_cast<const decltype(sut)*>(&sut))
    {
        EXPECT_THAT(v, Eq(INITIAL_VALUE + count));
        ++count;
    }
    EXPECT_THAT(count, Eq(EXPECTED_END_COUNT));
}

TEST_F(vector_test, IteratorIteratesThroughFullVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "147f78a9-0e60-43aa-ac72-c7a012904f5b");
    for (uint64_t k = 0U; k < VECTOR_CAPACITY; ++k)
    {
        sut.emplace_back(42U * k);
    }
    const auto EXPECTED_END_COUNT = sut.size();

    int i = 0;
    for (auto& v : sut)
    {
        EXPECT_THAT(v, Eq(42 * (i++)));
    }
    EXPECT_THAT(i, Eq(EXPECTED_END_COUNT));
}

TEST_F(vector_test, ConstIteratorIteratesThroughFullVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "0d8063b0-1a38-4130-a6cb-3e2a7f3c4304");
    for (uint64_t k = 0U; k < VECTOR_CAPACITY; ++k)
    {
        sut.emplace_back(142U * k);
    }
    const auto EXPECTED_END_COUNT = sut.size();

    int i = 0;
    // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    for (const auto& v : *const_cast<const decltype(sut)*>(&sut))
    {
        EXPECT_THAT(v, Eq(142 * (i++)));
    }
    EXPECT_THAT(i, Eq(EXPECTED_END_COUNT));
}

TEST_F(vector_test, IterateUsingData)
{
    ::testing::Test::RecordProperty("TEST_ID", "73d9a41f-3248-45a7-948e-2853c0ff8d3a");
    sut.emplace_back(127U);
    sut.emplace_back(128U);
    sut.emplace_back(129U);

    for (uint64_t k = 0U; k < sut.size(); ++k)
    {
        // NOLINTJUSTIFICATION Bounds considered
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic)
        EXPECT_THAT(sut.data()[k], Eq(127U + k));
    }
}

TEST_F(vector_test, IterateUsingConstData)
{
    ::testing::Test::RecordProperty("TEST_ID", "98feb75a-ba95-4598-a29a-8ac36840be3c");
    sut.emplace_back(3127U);
    sut.emplace_back(3128U);
    sut.emplace_back(3129U);

    for (uint64_t k = 0U; k < sut.size(); ++k)
    {
        // NOLINTJUSTIFICATION Bounds considered, const method tested
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic,cppcoreguidelines-pro-type-const-cast)
        EXPECT_THAT(const_cast<const decltype(sut)*>(&sut)->data()[k], Eq(3127U + k));
    }
}

TEST_F(vector_test, IterateUsingAt)
{
    ::testing::Test::RecordProperty("TEST_ID", "3d51b868-af12-47eb-bb0a-63ba87c8674a");
    sut.emplace_back(127U);
    sut.emplace_back(128U);
    sut.emplace_back(129U);

    for (uint64_t k = 0; k < sut.size(); ++k)
    {
        EXPECT_THAT(sut.at(k), Eq(127U + k));
    }
}

TEST_F(vector_test, IterateUsingConstAt)
{
    ::testing::Test::RecordProperty("TEST_ID", "65d76c52-0b7a-4b84-b5e3-eed51a887a6f");
    sut.emplace_back(3127U);
    sut.emplace_back(3128U);
    sut.emplace_back(3129U);

    for (uint64_t k = 0; k < sut.size(); ++k)
    {
        // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
        EXPECT_THAT(const_cast<const decltype(sut)*>(&sut)->at(k), Eq(3127U + k));
    }
}

TEST_F(vector_test, IterateUsingSquareBracket)
{
    ::testing::Test::RecordProperty("TEST_ID", "91eac70e-8555-4a37-bcd6-a0e4a41aff00");
    sut.emplace_back(2127U);
    sut.emplace_back(2128U);
    sut.emplace_back(2129U);

    for (uint64_t k = 0; k < sut.size(); ++k)
    {
        EXPECT_THAT(sut[k], Eq(2127U + k));
    }
}

TEST_F(vector_test, IterateUsingConstSquareBracket)
{
    ::testing::Test::RecordProperty("TEST_ID", "b1aa30bb-9aec-4419-8746-bba9b81d6049");
    sut.emplace_back(4127U);
    sut.emplace_back(4128U);
    sut.emplace_back(4129U);

    for (uint64_t k = 0; k < sut.size(); ++k)
    {
        // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
        EXPECT_THAT((*const_cast<const decltype(sut)*>(&sut))[k], Eq(4127U + k));
    }
}

TEST_F(vector_test, EraseFailsWhenElementIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "ff7c1c4a-4ef5-4905-a107-6f1d27462d47");
    auto* i = sut.begin() + 5U;
    EXPECT_FALSE(sut.erase(i));
    EXPECT_FALSE(sut.erase(sut.end()));
    EXPECT_FALSE(sut.erase(sut.begin() - 1));
}

TEST_F(vector_test, ErasingElementDecreasesSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "713074f9-0ad1-446e-a2a1-0707dcc112ca");
    sut.emplace_back(3U);
    sut.emplace_back(4U);
    sut.emplace_back(5U);
    EXPECT_TRUE(sut.erase(sut.begin() + 2U));
    EXPECT_TRUE(sut.erase(sut.begin()));
    EXPECT_THAT(sut.size(), Eq(1U));
}

TEST_F(vector_test, EraseOfLastElementCallsDTorOnly)
{
    ::testing::Test::RecordProperty("TEST_ID", "7af6f518-d95b-4643-87db-ec248be2cf8e");
    vector<CTorTest, 5U> sut1;
    sut1.emplace_back(7U);
    sut1.emplace_back(8U);
    sut1.emplace_back(9U);

    EXPECT_TRUE(sut1.erase(sut1.begin() + 2U));

    EXPECT_THAT(stats.dTor, Eq(1U));
    EXPECT_THAT(stats.classValue, Eq(9U));
}

TEST_F(vector_test, EraseOfMiddleElementCallsDTorAndMove)
{
    ::testing::Test::RecordProperty("TEST_ID", "caa4f0fb-3ddd-4273-9bec-66ef05a4c42b");
    vector<CTorTest, 5U> sut1;
    sut1.emplace_back(7U);
    sut1.emplace_back(8U);
    sut1.emplace_back(9U);
    sut1.emplace_back(10U);
    sut1.emplace_back(11U);

    EXPECT_TRUE(sut1.erase(sut1.begin() + 2U));

    EXPECT_THAT(stats.dTor, Eq(1U));
    EXPECT_THAT(stats.moveAssignment, Eq(2U));
}

TEST_F(vector_test, AccessOfNonExistingElementOnEmptyVectorLeadTermination)
{
    ::testing::Test::RecordProperty("TEST_ID", "31a4f0fb-31dd-4119-baba-31efab42c42b");

    ASSERT_THAT(sut.empty(), Eq(true));

    const uint64_t accessOffset{sut.size() + 1U};
    IOX_EXPECT_FATAL_FAILURE([&] { sut.at(accessOffset); }, iox::er::ENFORCE_VIOLATION);
}

TEST_F(vector_test, AccessOfNonExistingElementOnPartiallyFilledVectorLeadTermination)
{
    ::testing::Test::RecordProperty("TEST_ID", "13a1f2fb-01dd-3265-9bec-31ef0542c42b");
    constexpr int a{5};

    for (uint64_t i = 0U; i < (VECTOR_CAPACITY - 2U); ++i)
    {
        ASSERT_THAT(sut.push_back(a), Eq(true));
    }

    const uint64_t accessOffset{sut.size() + 1U};
    IOX_EXPECT_FATAL_FAILURE([&] { sut.at(accessOffset); }, iox::er::ENFORCE_VIOLATION);
}

TEST_F(vector_test, AccessOfNonExistingElementOnFullVectorLeadTermination)
{
    ::testing::Test::RecordProperty("TEST_ID", "42a4f0fb-71ad-1269-9b1c-71efca72c42b");
    constexpr int a{5};

    for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        ASSERT_THAT(sut.push_back(a), Eq(true));
    }

    ASSERT_THAT(sut.size(), Eq(VECTOR_CAPACITY));

    const uint64_t accessOffset{sut.size() + 1U};
    IOX_EXPECT_FATAL_FAILURE([&] { sut.at(accessOffset); }, iox::er::ENFORCE_VIOLATION);
}

TEST_F(vector_test, OutOfBoundsAccessOnEmptyVectorLeadsToTermination)
{
    ::testing::Test::RecordProperty("TEST_ID", "13d4f0fb-baba-1273-9b1c-acab15a4212b");

    ASSERT_THAT(sut.empty(), Eq(true));

    const uint64_t accessOffset{sut.size() + 1U};
    IOX_EXPECT_FATAL_FAILURE([&] { sut[accessOffset]; }, iox::er::ENFORCE_VIOLATION);
}

TEST_F(vector_test, OutOfBoundsAccessOnPartiallyFilledVectorLeadsToTermination)
{
    ::testing::Test::RecordProperty("TEST_ID", "59a4f0fb-ad31-c273-9b41-69153564242b");
    constexpr int a{5};

    for (uint64_t i = 0U; i < (VECTOR_CAPACITY - 2U); ++i)
    {
        ASSERT_THAT(sut.push_back(a), Eq(true));
    }

    const uint64_t accessOffset{sut.size() + 1U};
    IOX_EXPECT_FATAL_FAILURE([&] { sut[accessOffset]; }, iox::er::ENFORCE_VIOLATION);
}

TEST_F(vector_test, OutOfBoundsAccessOnFullVectorLeadsToTermination)
{
    ::testing::Test::RecordProperty("TEST_ID", "09a4fafa-3d31-3113-5bec-62ef01a4212b");
    constexpr int a{5};

    for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        ASSERT_THAT(sut.push_back(a), Eq(true));
    }

    const uint64_t accessOffset{sut.size() + 1U};
    IOX_EXPECT_FATAL_FAILURE([&] { sut[accessOffset]; }, iox::er::ENFORCE_VIOLATION);
}

TEST_F(vector_test, EraseOfFrontElementCallsDTorAndMove)
{
    ::testing::Test::RecordProperty("TEST_ID", "a5ce9c6f-0bc0-474b-9cff-5f9d317b4f95");
    vector<CTorTest, 5U> sut1;
    sut1.emplace_back(7U);
    sut1.emplace_back(8U);
    sut1.emplace_back(9U);
    sut1.emplace_back(10U);
    sut1.emplace_back(11U);

    EXPECT_TRUE(sut1.erase(sut1.begin()));

    EXPECT_THAT(stats.dTor, Eq(1U));
    EXPECT_THAT(stats.moveAssignment, Eq(4U));
}

TEST_F(vector_test, EraseMiddleElementDataCorrectAfterwards)
{
    ::testing::Test::RecordProperty("TEST_ID", "37448e1f-c069-4507-baa6-b66c0d47d4fc");
    sut.emplace_back(97U);
    sut.emplace_back(101U);
    sut.emplace_back(98U);
    sut.emplace_back(99U);

    EXPECT_TRUE(sut.erase(sut.begin() + 1U));

    for (uint64_t k = 0U; k < sut.size(); ++k)
    {
        EXPECT_THAT(sut[k], Eq(97U + k));
    }
}

TEST_F(vector_test, EraseFrontElementDataCorrectAfterwards)
{
    ::testing::Test::RecordProperty("TEST_ID", "5dea546e-16b6-4c48-b86a-ea86f334e7a7");
    sut.emplace_back(6101U);
    sut.emplace_back(597U);
    sut.emplace_back(598U);
    sut.emplace_back(599U);

    EXPECT_TRUE(sut.erase(sut.begin()));

    for (uint64_t k = 0U; k < sut.size(); ++k)
    {
        EXPECT_THAT(sut[k], Eq(597U + k));
    }
}

TEST_F(vector_test, EraseLastElementDataCorrectAfterwards)
{
    ::testing::Test::RecordProperty("TEST_ID", "f4c0a74a-de14-44e0-ac40-8875bd3c71d7");
    sut.emplace_back(7597U);
    sut.emplace_back(7598U);
    sut.emplace_back(7599U);
    sut.emplace_back(7600U);
    sut.emplace_back(7601U);
    sut.emplace_back(76101U);

    EXPECT_TRUE(sut.erase(sut.begin() + 5U));

    for (uint64_t k = 0U; k < sut.size(); ++k)
    {
        EXPECT_THAT(sut[k], Eq(7597U + k));
    }
}

TEST_F(vector_test, EraseLastElementOfFullVectorDataCorrectAfterwards)
{
    ::testing::Test::RecordProperty("TEST_ID", "fa4041c7-0fe4-43a9-8722-b1c6077b69d7");
    for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        sut.emplace_back(i * 123U);
    }

    EXPECT_TRUE(sut.erase(sut.begin() + sut.size() - 1U));

    for (uint64_t k = 0; k < sut.size(); ++k)
    {
        EXPECT_THAT(sut[k], Eq(k * 123U));
    }
}

TEST_F(vector_test, FrontPointsToFirstElement)
{
    ::testing::Test::RecordProperty("TEST_ID", "0f6283a1-16dd-41fb-892d-bb0b66805c1d");
    sut.emplace_back(1U);
    sut.emplace_back(2U);
    sut.emplace_back(3U);
    EXPECT_THAT(sut.front(), Eq(1));
}

TEST_F(vector_test, BackPointsToLastElement)
{
    ::testing::Test::RecordProperty("TEST_ID", "83cf678d-1e9e-4eb0-ac14-accb56b81d1b");
    sut.emplace_back(4U);
    sut.emplace_back(5U);
    sut.emplace_back(6U);
    EXPECT_THAT(sut.back(), Eq(6U));
}

TEST_F(vector_test, ConstFrontPointsToFirstElement)
{
    ::testing::Test::RecordProperty("TEST_ID", "34be7cb5-c9a8-42e7-b954-f37442e7ab54");
    sut.emplace_back(7U);
    sut.emplace_back(8U);
    sut.emplace_back(9U);

    // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    EXPECT_THAT(const_cast<const decltype(sut)*>(&sut)->front(), Eq(7U));
}

TEST_F(vector_test, ConstBackPointsToLastElement)
{
    ::testing::Test::RecordProperty("TEST_ID", "46be9689-0f94-42d2-94dc-36d1b768db24");
    sut.emplace_back(10U);
    sut.emplace_back(11U);
    sut.emplace_back(12U);

    // NOLINTJUSTIFICATION Re-use 'sut' and testing const methods
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    EXPECT_THAT(const_cast<const decltype(sut)*>(&sut)->back(), Eq(12U));
}

TEST_F(vector_test, ConstructorWithSizeParameterSmallerThanCapacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "b55f3818-ded5-420a-ad9a-88d5e90b429e");
    vector<CTorTest, 5U> sut(2U);
    EXPECT_THAT(stats.cTor, Eq(2U));
    ASSERT_THAT(sut.size(), Eq(2U));
}

TEST_F(vector_test, ConstructorWithSizeParameterGreaterThanCapacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "57d86dd4-ba23-4911-a451-bbc78d3f899a");
    vector<CTorTest, 5U> sut(7U);
    EXPECT_THAT(stats.cTor, Eq(5U));
    ASSERT_THAT(sut.size(), Eq(5U));
}

TEST_F(vector_test, TwoEmptyVectorOfSameCapacityAreEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "80020e56-7cc1-4fbc-9e4f-aecac5fb6110");
    vector<int, 10> a;
    vector<int, 10> b;

    EXPECT_TRUE(a == b);
    EXPECT_FALSE(a != b);
}

TEST_F(vector_test, TwoEmptyVectorOfDifferentCapacityAreEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "45c18d99-4c2b-4e3f-9ac0-e3277bb5946a");
    vector<int, 10> a;
    vector<int, 20> b;

    EXPECT_TRUE(a == b);
    EXPECT_FALSE(a != b);
}

TEST_F(vector_test, TwoEqualVectorsWithSameCapacityAreEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "15d18e5b-afd5-4d81-82c1-146c7d11760f");
    vector<int, 10> a;
    vector<int, 10> b;

    a.emplace_back(1);
    a.emplace_back(2);
    a.emplace_back(3);
    b.emplace_back(1);
    b.emplace_back(2);
    b.emplace_back(3);

    EXPECT_TRUE(a == b);
    EXPECT_FALSE(a != b);
}

TEST_F(vector_test, TwoEqualVectorsWithDifferentCapacityAreEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "b3f5b902-1635-41da-aeba-1b469dea97d8");
    vector<int, 10> a;
    vector<int, 20> b;

    a.emplace_back(4);
    a.emplace_back(5);
    a.emplace_back(6);
    b.emplace_back(4);
    b.emplace_back(5);
    b.emplace_back(6);

    EXPECT_TRUE(a == b);
    EXPECT_FALSE(a != b);
}

TEST_F(vector_test, TwoVectorsWithDifferentSizeAndSameCapacityAreNotEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "561722c3-14c3-43a4-aebc-c92bc6be5c08");
    vector<int, 10> a;
    vector<int, 10> b;

    a.emplace_back(7);
    a.emplace_back(8);
    a.emplace_back(9);

    EXPECT_FALSE(a == b);
    EXPECT_TRUE(a != b);
}

TEST_F(vector_test, TwoNonEqualVectorsWithDifferentCapacityAreNotEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "2a8a2ed9-ec6c-4790-aac5-c9e397a78532");
    vector<int, 10> a;
    a.emplace_back(7);
    a.emplace_back(8);
    a.emplace_back(9);

    vector<int, 20> b;
    b.emplace_back(1);
    b.emplace_back(2);
    b.emplace_back(3);

    EXPECT_FALSE(a == b);
    EXPECT_TRUE(a != b);
}

TEST_F(vector_test, SubsetVectorWithSameCapacityIsNotEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "dd1ebe03-e503-4e18-949a-d620ce8008ae");
    vector<int, 10> a;
    vector<int, 10> b;
    a.emplace_back(7);
    a.emplace_back(8);
    a.emplace_back(9);

    b.emplace_back(7);
    b.emplace_back(8);

    EXPECT_FALSE(a == b);
    EXPECT_TRUE(a != b);
}

TEST_F(vector_test, SubsetVectorWithDifferentCapacityIsNotEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "f176f89e-7505-4d67-ba83-9834d2737c8f");
    vector<int, 10> a;
    a.emplace_back(11);
    a.emplace_back(12);
    a.emplace_back(13);

    vector<int, 20> b;
    b.emplace_back(11);
    b.emplace_back(12);

    EXPECT_FALSE(a == b);
    EXPECT_TRUE(a != b);
}

TEST_F(vector_test, PartiallyEqualVectorsWithSameCapacityAreNotEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "5087abfd-a3d6-4312-b4f6-ba7a0db0a4a8");
    vector<int, 10> a;
    vector<int, 10> b;

    a.emplace_back(14);
    a.emplace_back(15);
    a.emplace_back(16);
    b.emplace_back(14);
    b.emplace_back(15);
    b.emplace_back(666);

    EXPECT_FALSE(a == b);
    EXPECT_TRUE(a != b);
}

TEST_F(vector_test, PartiallyEqualVectorsWithDifferentCapacityAreNotEqual)
{
    ::testing::Test::RecordProperty("TEST_ID", "4291adad-b67d-4382-8b17-ba3f6ffed480");
    vector<int, 10> a;
    a.emplace_back(17);
    a.emplace_back(18);
    a.emplace_back(19);

    vector<int, 20> b;
    b.emplace_back(17);
    b.emplace_back(18);
    b.emplace_back(999);

    EXPECT_FALSE(a == b);
    EXPECT_TRUE(a != b);
}

TEST_F(vector_test, FullVectorDestroysElementsInReverseOrder)
{
    ::testing::Test::RecordProperty("TEST_ID", "16b26245-05d0-458f-82d2-6946d2f8ba07");
    static constexpr uint64_t VECTOR_CAPACITY = 35U;
    static constexpr uint64_t INDEX_END = VECTOR_CAPACITY - 1U;
    static constexpr uint64_t SOME_OFFSET = 9128U;

    {
        vector<CTorTest, VECTOR_CAPACITY> sut;

        for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
        {
            sut.emplace_back(i + SOME_OFFSET);
        }
    }

    ASSERT_THAT(stats.dTorOrder.size(), Eq(VECTOR_CAPACITY));
    for (size_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        EXPECT_THAT(stats.dTorOrder[i], Eq(INDEX_END - i + SOME_OFFSET));
    }
}

TEST_F(vector_test, PartiallyFullVectorDestroysElementsInReverseOrder)
{
    ::testing::Test::RecordProperty("TEST_ID", "bd1f5c02-0636-4fdc-a369-f61439cd2e3e");
    static constexpr uint64_t VECTOR_CAPACITY = 40U;
    static constexpr uint64_t VECTOR_SIZE = 20U;
    static constexpr uint64_t INDEX_END = VECTOR_SIZE - 1U;
    static constexpr uint64_t SOME_OFFSET = 1337U;

    {
        vector<CTorTest, VECTOR_CAPACITY> sut;

        for (uint64_t i = 0U; i < VECTOR_SIZE; ++i)
        {
            sut.emplace_back(i + SOME_OFFSET);
        }
    }

    ASSERT_THAT(stats.dTorOrder.size(), Eq(VECTOR_SIZE));
    for (size_t i = 0U; i < VECTOR_SIZE; ++i)
    {
        EXPECT_THAT(stats.dTorOrder[i], Eq(INDEX_END - i + SOME_OFFSET));
    }
}

TEST_F(vector_test, PopBackReturnsFalseOnEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "d11cf587-39cb-4024-be26-d76ada767686");
    EXPECT_FALSE(sut.pop_back());
}

TEST_F(vector_test, PopBackReturnsTrueOnNonEmptyVector)
{
    ::testing::Test::RecordProperty("TEST_ID", "7c09370e-f656-4d30-a7c3-f79a935c7aef");
    sut.emplace_back(123U);
    EXPECT_TRUE(sut.pop_back());
}

TEST_F(vector_test, PopBackReturnsTrueTillItsEmpty)
{
    ::testing::Test::RecordProperty("TEST_ID", "5c0f98b3-3ca1-43dd-9855-2e9e867ee5c4");
    static constexpr uint64_t VECTOR_SIZE = 5U;
    for (uint64_t i = 0U; i < VECTOR_SIZE; ++i)
    {
        sut.emplace_back(i);
    }

    for (uint64_t i = 0U; i < VECTOR_SIZE; ++i)
    {
        EXPECT_TRUE(sut.pop_back());
    }

    EXPECT_FALSE(sut.pop_back());
}

TEST_F(vector_test, ResizeFailsWhenCountIsGreaterThanCapacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "52b98fee-ca67-465c-853f-8df88d4a572d");
    EXPECT_FALSE(sut.resize(sut.capacity() + 1U));
}

TEST_F(vector_test, ResizeWithTemplateValueFailsWhenCountIsGreaterThanCapacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "525c2de0-a6ed-4c8d-8c5e-ab974bada3e4");
    EXPECT_FALSE(sut.resize(sut.capacity() + 1U, 12U));
}

TEST_F(vector_test, SizeIncreaseWithResizeAndDefaultCTorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "c7e665f9-c051-437b-af91-944d40719da4");
    class DefaultCTor
    {
      public:
        DefaultCTor() = default;
        int m_a{1231};
    };
    vector<DefaultCTor, 10U> sut;

    EXPECT_TRUE(sut.resize(5U));
    ASSERT_THAT(sut.size(), Eq(5U));
    for (auto& e : sut)
    {
        EXPECT_THAT(e.m_a, Eq(1231));
    }
}

TEST_F(vector_test, SizeIncreaseWithResizeAndTemplateValueWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "5dc23a28-177c-4e0b-a454-e2b09acba11f");
    EXPECT_TRUE(sut.resize(4U, 421337U));
    ASSERT_THAT(sut.size(), Eq(4U));
    for (auto& e : sut)
    {
        EXPECT_THAT(e, Eq(421337));
    }
}

TEST_F(vector_test, SizeDecreaseWithResizeAndDefaultCTorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "bfd86fcc-c828-4b1b-ab9a-cff7e0f22164");
    constexpr uint64_t CAPACITY{10};
    vector<CTorTest, CAPACITY> sut;
    for (uint64_t i = 0U; i < CAPACITY; ++i)
    {
        sut.emplace_back(i);
    }

    EXPECT_TRUE(sut.resize(7U));
    EXPECT_THAT(stats.dTor, Eq(3U));
    ASSERT_THAT(stats.dTorOrder.size(), Eq(3U));
    EXPECT_THAT(stats.dTorOrder[0], Eq(9));
    EXPECT_THAT(stats.dTorOrder[1], Eq(8));
    EXPECT_THAT(stats.dTorOrder[2], Eq(7));
}

TEST_F(vector_test, SizeDecreaseWithResizeAndTemplateValueWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "6b2d81ce-1d46-47a6-bbb2-16f1c0ce46f3");
    constexpr uint64_t CAPACITY{10};
    vector<CTorTest, CAPACITY> sut;
    for (uint64_t i = 0U; i < CAPACITY; ++i)
    {
        sut.emplace_back(i + 10U);
    }

    EXPECT_TRUE(sut.resize(7U, 66807U));
    EXPECT_THAT(stats.dTor, Eq(3U));
    ASSERT_THAT(stats.dTorOrder.size(), Eq(3U));
    EXPECT_THAT(stats.dTorOrder[0], Eq(19));
    EXPECT_THAT(stats.dTorOrder[1], Eq(18));
    EXPECT_THAT(stats.dTorOrder[2], Eq(17));
}

TEST_F(vector_test, ResizeWithDefaultCTorChangesNothingIfSizeAlreadyFits)
{
    ::testing::Test::RecordProperty("TEST_ID", "7d4bf455-a7b4-4bb5-87f1-53d9552b91d5");
    sut.emplace_back(5U);
    sut.emplace_back(6U);
    EXPECT_TRUE(sut.resize(2U));

    ASSERT_THAT(sut.size(), Eq(2U));
    EXPECT_THAT(sut[0], Eq(5U));
    EXPECT_THAT(sut[1], Eq(6U));
}

TEST_F(vector_test, ResizeWithTemplateValueChangesNothingIfSizeAlreadyFits)
{
    ::testing::Test::RecordProperty("TEST_ID", "6ccd9ac4-788c-428b-9d11-37885284088f");
    sut.emplace_back(7U);
    sut.emplace_back(9U);
    EXPECT_TRUE(sut.resize(2U, 421337U));

    ASSERT_THAT(sut.size(), Eq(2U));
    EXPECT_THAT(sut[0], Eq(7U));
    EXPECT_THAT(sut[1], Eq(9U));
}

TEST_F(vector_test, EmplaceInEmptyVectorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "e6b1b8d4-77b6-4a19-8d7e-7f483e2e461d");
    EXPECT_TRUE(sut.emplace(0U, 123U));
    ASSERT_THAT(sut.size(), Eq(1U));
    EXPECT_THAT(sut[0], Eq(123U));
}

TEST_F(vector_test, EmplaceAtFrontTillFullWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "c7074b38-8493-4b53-acc2-9a20d0f735ce");
    for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        EXPECT_TRUE(sut.emplace(0U, i));
        ASSERT_THAT(sut.size(), Eq(i + 1U));

        for (uint64_t n = 0U; n < sut.size(); ++n)
        {
            EXPECT_THAT(sut[n], Eq(sut.size() - n - 1U));
        }
    }
}

TEST_F(vector_test, EmplaceInTheMiddleMovesElementsToTheRight)
{
    ::testing::Test::RecordProperty("TEST_ID", "ab181814-6743-43a2-8420-c725b3afd800");
    sut.emplace_back(0U);
    sut.emplace_back(1U);
    sut.emplace_back(2U);

    EXPECT_TRUE(sut.emplace(1U, 3U));

    ASSERT_THAT(sut.size(), Eq(4));

    EXPECT_THAT(sut[0], Eq(0U));
    EXPECT_THAT(sut[1], Eq(3U));
    EXPECT_THAT(sut[2], Eq(1U));
    EXPECT_THAT(sut[3], Eq(2U));
}

TEST_F(vector_test, EmplaceWhenFullReturnsFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "93e5d45c-9450-4ceb-8d1c-78aae413eca8");
    for (uint64_t i = 0U; i < VECTOR_CAPACITY; ++i)
    {
        sut.emplace_back(i);
    }

    auto index = VECTOR_CAPACITY / 2;
    EXPECT_FALSE(sut.emplace(index, 5U));
    EXPECT_THAT(sut.size(), Eq(sut.capacity()));
}

TEST_F(vector_test, EmplaceWhenPositionExceedsCapacityReturnsFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "519d97fb-aec0-4824-9cd7-dd3446b7b71c");
    EXPECT_FALSE(sut.emplace(sut.capacity() + 10U, 5U));
    EXPECT_THAT(sut.size(), Eq(0));
}

TEST_F(vector_test, EmplaceAtEndWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "57551774-750f-4dd4-81c0-fa6ef9046689");
    sut.emplace_back(0U);
    sut.emplace_back(1U);

    EXPECT_TRUE(sut.emplace(sut.size(), 3U));
    ASSERT_THAT(sut.size(), Eq(3U));
    EXPECT_THAT(sut[0], Eq(0U));
    EXPECT_THAT(sut[1], Eq(1U));
    EXPECT_THAT(sut[2], Eq(3U));
}

TEST_F(vector_test, EmplaceAtPositionAfterEndBeforeCapacityExceedsFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "b5112070-9446-44bf-8fdf-1853cfb247fc");
    sut.emplace_back(0U);
    sut.emplace_back(1U);

    constexpr uint64_t EXPECTED_SIZE{2};
    ASSERT_THAT(sut.size(), EXPECTED_SIZE);
    EXPECT_FALSE(sut.emplace(EXPECTED_SIZE + 1, 3U));
    ASSERT_THAT(sut.size(), EXPECTED_SIZE);
}
} // namespace
