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

#include "iox/algorithm.hpp"
#include "iox/vector.hpp"
#include "test.hpp"

namespace
{
using namespace ::testing;
using namespace iox::algorithm;
using namespace iox;

class algorithm_test : public Test
{
  public:
    void SetUp() override
    {
    }
    void TearDown() override
    {
    }
};

TEST_F(algorithm_test, MaxOfOneElement)
{
    ::testing::Test::RecordProperty("TEST_ID", "3fba70b4-252b-4c13-a98c-87b026254bba");
    EXPECT_THAT(maxVal(12.34F), Eq(12.34F));
}

TEST_F(algorithm_test, MaxOfTwoElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "0443931f-3eb4-4ae2-99b3-029637f94d0f");
    EXPECT_THAT(maxVal(56.78F, 12.34F), Eq(56.78F));
}

TEST_F(algorithm_test, MaxOfManyElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "83c16bb2-90c5-4226-bed2-7e5cc5b34f22");
    EXPECT_THAT(maxVal(56.78F, 33.44F, 12.34F, -0.1F, 5.5F, 10001.F), Eq(10001.F));
}

TEST_F(algorithm_test, MinOfOneElement)
{
    ::testing::Test::RecordProperty("TEST_ID", "384d8139-1a79-40ae-8caf-b468470c48d2");
    EXPECT_THAT(minVal(0.0123F), Eq(0.0123F));
}

TEST_F(algorithm_test, MinOfTwoElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "c0ad7d53-03f6-4ee2-9a0b-ee929dc047a7");
    EXPECT_THAT(minVal(0.0123F, -91.12F), Eq(-91.12F));
}

TEST_F(algorithm_test, MinOfManyElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "8ec6db69-2260-4af9-83fe-73ae58c878b3");
    EXPECT_THAT(minVal(0.0123F, -91.12F, 123.92F, -1021.2F, 0.0F), Eq(-1021.2F));
}

TEST_F(algorithm_test, DoesContainValue_ValueListOfZeroDoesNotContainValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "b8ef3cdf-8cfa-469c-ac67-7fc4afbc9b64");
    EXPECT_FALSE(doesContainValue(42));
}

TEST_F(algorithm_test, DoesContainValue_ValueListOfOneDoesNotContainValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "9836ad33-da77-476a-a074-1cf7878bcbe6");
    EXPECT_FALSE(doesContainValue(37, 13));
}

TEST_F(algorithm_test, DoesContainValue_ValueListOfOneDoesContainValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "643c842c-2dd2-4741-b344-b58fa5e32a6a");
    EXPECT_TRUE(doesContainValue(73, 73));
}

TEST_F(algorithm_test, DoesContainValue_ValueListOfMultipleValuesDoesNotContainValue)
{
    ::testing::Test::RecordProperty("TEST_ID", "e0131b57-51b9-439f-a372-3725bfa7f24a");
    EXPECT_FALSE(doesContainValue(13, 42, 73, 7337));
}

TEST_F(algorithm_test, DoesContainValue_ValueListOfMultipleValuesDoesContainValueAtFront)
{
    ::testing::Test::RecordProperty("TEST_ID", "ad37f570-e20d-46ca-85ef-a62fdbfeb9c2");
    EXPECT_TRUE(doesContainValue(37, 37, 3773, 7535));
}

TEST_F(algorithm_test, DoesContainValue_ValueListOfMultipleValuesDoesContainValueInTheMiddle)
{
    ::testing::Test::RecordProperty("TEST_ID", "bbc397c4-5d15-4acf-a317-b93a6537571c");
    EXPECT_TRUE(doesContainValue(42, 13, 42, 555));
}

TEST_F(algorithm_test, DoesContainValue_ValueListOfMultipleValuesDoesContainValueAtEnd)
{
    ::testing::Test::RecordProperty("TEST_ID", "64c87a80-e83b-4e70-8f76-476f24804f19");
    EXPECT_TRUE(doesContainValue(7353, 42, 73, 7353));
}
TEST_F(algorithm_test, BestFittingTypeUsesUint8WhenValueSmaller256)
{
    ::testing::Test::RecordProperty("TEST_ID", "6704aaf9-c0a4-495c-8128-15c126cbcd9b");
    EXPECT_TRUE((std::is_same<BestFittingType_t<123U>, uint8_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint8WhenValueEqualTo255)
{
    ::testing::Test::RecordProperty("TEST_ID", "10bbca50-95a7-436b-ab54-43b37cc7048f");
    EXPECT_TRUE((std::is_same<BestFittingType_t<255U>, uint8_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint16WhenValueEqualTo256)
{
    ::testing::Test::RecordProperty("TEST_ID", "d67306ff-c0cc-4769-9160-ef14e9f482dc");
    EXPECT_TRUE((std::is_same<BestFittingType_t<256U>, uint16_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint16WhenValueBetween256And65535)
{
    ::testing::Test::RecordProperty("TEST_ID", "ff50f669-d9d3-454f-9994-a4dd3a19029d");
    EXPECT_TRUE((std::is_same<BestFittingType_t<8172U>, uint16_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint16WhenValueEqualTo65535)
{
    ::testing::Test::RecordProperty("TEST_ID", "b71d99b4-bd4e-46d6-8b22-6e796b611824");
    EXPECT_TRUE((std::is_same<BestFittingType_t<65535U>, uint16_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint32WhenValueEqualTo65536)
{
    ::testing::Test::RecordProperty("TEST_ID", "fe53df8e-a797-4547-8503-0ff5850ab22e");
    EXPECT_TRUE((std::is_same<BestFittingType_t<65536U>, uint32_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint32WhenValueBetween2p16And2p32)
{
    ::testing::Test::RecordProperty("TEST_ID", "f07b1301-faf1-4945-aab0-a7af0ac967d7");
    EXPECT_TRUE((std::is_same<BestFittingType_t<81721U>, uint32_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint32WhenValueEqualTo4294967295)
{
    ::testing::Test::RecordProperty("TEST_ID", "f63335ef-c29f-49f0-bd77-ea9a548ef9fa");
    EXPECT_TRUE((std::is_same<BestFittingType_t<4294967295U>, uint32_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint64WhenValueEqualTo4294967296)
{
    ::testing::Test::RecordProperty("TEST_ID", "23f6ff5c-4cad-440c-839f-bd6cde5fa5d4");
    EXPECT_TRUE((std::is_same<BestFittingType_t<4294967296U>, uint64_t>::value));
}

TEST_F(algorithm_test, BestFittingTypeUsesUint32WhenValueGreater2p32)
{
    ::testing::Test::RecordProperty("TEST_ID", "8fddfb4c-0efb-4b21-9b15-8f49af779f84");
    EXPECT_TRUE((std::is_same<BestFittingType_t<42949672961U>, uint64_t>::value));
}

template <class T>
class algorithm_test_isPowerOfTwo : public algorithm_test
{
  public:
    using CurrentType = T;

    static constexpr T MAX = std::numeric_limits<T>::max();
    static constexpr T MAX_POWER_OF_TWO = MAX / 2U + 1U;
};
using HelpletsIsPowerOfTwoTypes = Types<uint8_t, uint16_t, uint32_t, uint64_t, size_t>;
TYPED_TEST_SUITE(algorithm_test_isPowerOfTwo, HelpletsIsPowerOfTwoTypes, );

TYPED_TEST(algorithm_test_isPowerOfTwo, OneIsPowerOfTwo)
{
    ::testing::Test::RecordProperty("TEST_ID", "c85e1998-436c-4789-95c5-895fe7b2edf0");
    EXPECT_TRUE(isPowerOfTwo(static_cast<typename TestFixture::CurrentType>(1)));
}

TYPED_TEST(algorithm_test_isPowerOfTwo, TwoIsPowerOfTwo)
{
    ::testing::Test::RecordProperty("TEST_ID", "6d314d4b-1206-4779-9035-fa544cfee798");
    EXPECT_TRUE(isPowerOfTwo(static_cast<typename TestFixture::CurrentType>(2)));
}

TYPED_TEST(algorithm_test_isPowerOfTwo, FourIsPowerOfTwo)
{
    ::testing::Test::RecordProperty("TEST_ID", "cb2ad241-4515-4bfb-8078-157ed8c0e18d");
    EXPECT_TRUE(isPowerOfTwo(static_cast<typename TestFixture::CurrentType>(4)));
}

TYPED_TEST(algorithm_test_isPowerOfTwo, MaxPossiblePowerOfTwoForTypeIsPowerOfTwo)
{
    ::testing::Test::RecordProperty("TEST_ID", "b92311dd-aa33-489d-8544-6054028c35a4");
    EXPECT_TRUE(isPowerOfTwo(static_cast<typename TestFixture::CurrentType>(TestFixture::MAX_POWER_OF_TWO)));
}

TYPED_TEST(algorithm_test_isPowerOfTwo, ZeroIsNotPowerOfTwo)
{
    ::testing::Test::RecordProperty("TEST_ID", "6a8295cd-664d-4b1f-8a20-ac814c7f75c5");
    EXPECT_FALSE(isPowerOfTwo(static_cast<typename TestFixture::CurrentType>(0)));
}

TYPED_TEST(algorithm_test_isPowerOfTwo, FourtyTwoIsNotPowerOfTwo)
{
    ::testing::Test::RecordProperty("TEST_ID", "0570fc10-eb72-4a34-b8a6-5084c7737866");
    EXPECT_FALSE(isPowerOfTwo(static_cast<typename TestFixture::CurrentType>(42)));
}

TYPED_TEST(algorithm_test_isPowerOfTwo, MaxValueForTypeIsNotPowerOfTwo)
{
    ::testing::Test::RecordProperty("TEST_ID", "2abdb27d-58de-4e3d-b8fb-8e5f1f3e6327");
    EXPECT_FALSE(isPowerOfTwo(static_cast<typename TestFixture::CurrentType>(TestFixture::MAX)));
}
} // namespace
