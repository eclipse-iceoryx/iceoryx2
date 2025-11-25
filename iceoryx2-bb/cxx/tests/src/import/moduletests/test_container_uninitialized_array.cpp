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

#include "test.hpp"

#include "iox/uninitialized_array.hpp"

namespace
{
using namespace ::testing;

using iox::UninitializedArray;

struct Integer
{
    static uint32_t ctor;
    static uint32_t dtor;

    Integer()
        : Integer(0)
    {
    }

    // NOLINTNEXTLINE(hicpp-explicit-conversions) required for typed tests
    Integer(int value)
        : value(value)
    {
        ctor++;
    }

    ~Integer()
    {
        dtor++;
    }

    Integer(const Integer&) = delete;
    Integer(Integer&&) = delete;
    Integer& operator=(const Integer&) = delete;
    Integer& operator=(Integer&&) = delete;

    int value{0};

    // so that it behaves like an int for comparison purposes
    // NOLINTNEXTLINE(hicpp-explicit-conversions) required for typed tests
    operator int() const
    {
        return value;
    }
};

uint32_t Integer::ctor;
uint32_t Integer::dtor;

template <typename T>
class UninitializedArrayTest : public ::testing::Test
{
  public:
    using Buffer = T;
    using Type = typename Buffer::value_type;
    Buffer buffer{};

    void fillBuffer(int startValue)
    {
        auto capacity = buffer.capacity();

        int value = startValue;
        for (uint64_t i = 0; i < capacity; ++i)
        {
            new (&buffer[i]) Type(value++);
        }
    }

    void SetUp() override
    {
        Integer::ctor = 0;
        Integer::dtor = 0;
    }
};

TEST(UninitializedArrayTest, capacityIsCorrect)
{
    ::testing::Test::RecordProperty("TEST_ID", "0ac31a08-77b2-4fd2-a214-81298cada00c");
    constexpr uint64_t capacity = 7;
    // NOLINTNEXTLINE(hicpp-member-init, cppcoreguidelines-pro-type-member-init) false positive
    UninitializedArray<int, capacity> sut;
    EXPECT_EQ(sut.capacity(), capacity);
}

TEST(UninitializedArrayTest, isNotCopyConstructible)
{
    ::testing::Test::RecordProperty("TEST_ID", "abc31a08-77b2-4fd2-af14-3129bafda00c");
    constexpr uint64_t CAPACITY{31U};

    bool isCopyConstructible = std::is_copy_constructible<UninitializedArray<int, CAPACITY>>::value;
    EXPECT_FALSE(isCopyConstructible);
}

TEST(UninitializedArrayTest, isNotCopyAssignable)
{
    ::testing::Test::RecordProperty("TEST_ID", "42c31a08-77b2-4fd2-6914-3129869da00c");
    constexpr uint64_t CAPACITY{69U};

    bool isCopyAssignable = std::is_copy_assignable<UninitializedArray<int, CAPACITY>>::value;
    EXPECT_FALSE(isCopyAssignable);
}


TEST(UninitializedArrayTest, isNotMoveConstructible)
{
    ::testing::Test::RecordProperty("TEST_ID", "baf31a08-77b2-4692-6914-31298693100c");
    constexpr uint64_t CAPACITY{13U};

    bool isMoveConstructible = std::is_move_constructible<UninitializedArray<int, CAPACITY>>::value;
    EXPECT_FALSE(isMoveConstructible);
}

TEST(UninitializedArrayTest, isNotMoveAssignable)
{
    ::testing::Test::RecordProperty("TEST_ID", "caba1a08-77b2-4fd2-3114-3129842daa0c");
    constexpr uint64_t CAPACITY{42U};

    bool isMoveAssignable = std::is_move_assignable<UninitializedArray<int, CAPACITY>>::value;
    EXPECT_FALSE(isMoveAssignable);
}

typedef ::testing::Types<UninitializedArray<int, 10>,
                         UninitializedArray<Integer, 10>,
                         UninitializedArray<int, 10, iox::ZeroedBuffer>,
                         UninitializedArray<Integer, 10, iox::ZeroedBuffer>>
    TestArrays;

TYPED_TEST_SUITE(UninitializedArrayTest, TestArrays, );

TYPED_TEST(UninitializedArrayTest, accessElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "f8621711-9c4c-45b5-979d-404357b664a4");
    auto& buffer = this->buffer;
    auto capacity = buffer.capacity();

    int startValue = 73;
    this->fillBuffer(startValue);

    int expected = startValue;
    for (uint64_t i = 0; i < capacity; ++i)
    {
        EXPECT_EQ(buffer[i], expected++);
    }
}

TYPED_TEST(UninitializedArrayTest, accessElementsOfConstUinitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "50575bac-cff9-4896-89da-b03753370b18");
    auto& buffer = this->buffer;
    auto capacity = buffer.capacity();

    int startValue = 37;
    this->fillBuffer(startValue);

    const auto& constBuffer = this->buffer;
    int expected = startValue;
    for (uint64_t i = 0; i < capacity; ++i)
    {
        EXPECT_EQ(constBuffer[i], expected++);
    }
}

TEST(UninitializedArrayTest, AllElementsInitializedWithZeroWhenBufferSetToZeroedBuffer)
{
    ::testing::Test::RecordProperty("TEST_ID", "bb213516-ab37-43e3-b2ec-098c98d777d1");
    constexpr uint64_t CAPACITY{32};
    UninitializedArray<uint32_t, CAPACITY, iox::ZeroedBuffer> buffer;
    for (auto& e : buffer)
    {
        new (&e) uint32_t(std::numeric_limits<uint32_t>::max());
        // Access 'e' to prevent the compiler from optimizing away the loop
        EXPECT_EQ(e, std::numeric_limits<uint32_t>::max());
    }

    new (&buffer) UninitializedArray<uint32_t, CAPACITY, iox::ZeroedBuffer>();

    for (auto& e : buffer)
    {
        EXPECT_EQ(e, 0);
    }
}

TEST(UninitializedArrayTest, AllElementsAreNotZeroedWhenBufferSetToNonZeroedBuffer)
{
    ::testing::Test::RecordProperty("TEST_ID", "35666437-6ee5-4940-b053-e82d8e312a11");
    constexpr uint64_t CAPACITY{32};
    UninitializedArray<uint32_t, CAPACITY, iox::ZeroedBuffer> buffer;
    for (auto& e : buffer)
    {
        new (&e) uint32_t(std::numeric_limits<uint32_t>::max());
        // Access 'e' to prevent the compiler from optimizing away the loop
        EXPECT_EQ(e, std::numeric_limits<uint32_t>::max());
    }

    new (&buffer) UninitializedArray<uint32_t, CAPACITY, iox::NonZeroedBuffer>();

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays) : explicitly required for test
    uint32_t result[CAPACITY];
    memcpy(&result, &buffer, CAPACITY * sizeof(uint32_t));

    for (uint64_t i = 0; i < CAPACITY; i++)
    {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index) : false positive, is integer constexpr
        EXPECT_EQ(result[i], std::numeric_limits<uint32_t>::max());
    }
}

TYPED_TEST(UninitializedArrayTest, BeginReturnsIteratorToBeginningOfUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "6434e308-e24f-41e1-a1e1-949da01b2cbb");
    auto& buffer = this->buffer;
    EXPECT_EQ(buffer.begin(), &buffer[0]);
}

TYPED_TEST(UninitializedArrayTest, ConstBeginReturnsIteratorToBeginningOfUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "7387b043-db44-47ac-a2da-c40040bb9baa");
    auto& buffer = this->buffer;
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) reuse of buffer to test const method
    EXPECT_EQ(const_cast<const decltype(buffer)>(buffer).begin(), &buffer[0]);
}

TYPED_TEST(UninitializedArrayTest, EndReturnsIteratorToEndOfUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "52447fba-0c7f-40df-8b7f-64d8b3ffcc49");
    auto& buffer = this->buffer;
    EXPECT_EQ(buffer.end(), &buffer[buffer.capacity()]);
}

TYPED_TEST(UninitializedArrayTest, ConstEndReturnsIteratorToEndOfUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "2946ad83-b782-4c54-966b-c94b482335cc");
    auto& buffer = this->buffer;
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) reuse of buffer to test const method
    EXPECT_EQ(const_cast<const decltype(buffer)>(buffer).end(), &buffer[buffer.capacity()]);
}

TEST(UninitializedArrayTest, BeginAndEndIteratorNotEqualInNonEmptyUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "1a180d15-7e77-4234-ad88-04673cbf9fc9");
    constexpr uint64_t CAPACITY = 3;
    using Buffer = UninitializedArray<uint32_t, CAPACITY>;
    Buffer buffer;

    new (&buffer[0]) Buffer::value_type(1);

    EXPECT_NE(buffer.begin(), buffer.end());
}

TEST(UninitializedArrayTest, BeginAndEndConstIteratorNotEqualInNonEmptyUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "33e3d5b4-4762-421a-829f-455fe44b8e3b");
    constexpr uint64_t CAPACITY = 3;
    using Buffer = UninitializedArray<int32_t, CAPACITY>;
    Buffer buffer;

    new (&buffer[0]) Buffer::value_type(2);

    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) reuse of buffer to test const methods
    EXPECT_NE(const_cast<const Buffer&>(buffer).begin(), const_cast<const Buffer&>(buffer).end());
}

TYPED_TEST(UninitializedArrayTest, BeginAndEndIteratorNotEqualInFullUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "2ac41459-1055-47c0-8dc1-ea43c50827bf");
    auto& buffer = this->buffer;
    this->fillBuffer(0);
    EXPECT_NE(buffer.begin(), buffer.end());
}

TYPED_TEST(UninitializedArrayTest, BeginAndEndConstIteratorNotEqualInFullUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "01a5d1cd-ba6d-422e-a807-1ffe5787f4af");
    auto& buffer = this->buffer;
    this->fillBuffer(2);
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) reuse of buffer to test const methods
    EXPECT_NE(const_cast<const decltype(buffer)>(buffer).begin(), const_cast<const decltype(buffer)>(buffer).end());
}

TEST(UninitializedArrayTest, IteratorIteratesThroughUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "b42d93c9-cbe8-481f-8a0b-5b3fb8e9020c");
    constexpr uint64_t CAPACITY = 3;
    constexpr uint32_t INITIAL_VALUE = 42U;

    using Buffer = UninitializedArray<uint32_t, CAPACITY>;
    using Type = Buffer::value_type;
    Buffer buffer;

    new (&buffer[0]) Type(INITIAL_VALUE);
    new (&buffer[1]) Type(INITIAL_VALUE + 1);
    new (&buffer[2]) Type(INITIAL_VALUE + 2);

    uint32_t count{0};
    for (auto& e : buffer)
    {
        EXPECT_EQ(e, INITIAL_VALUE + count);
        ++count;
    }
    EXPECT_EQ(count, CAPACITY);
}

TEST(UninitializedArrayTest, ConstIteratorIteratesThroughUninitializedArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "e8d7ac7f-9ec7-4264-8b27-d0469b167375");
    constexpr uint64_t CAPACITY = 3;
    constexpr uint32_t INITIAL_VALUE = 13;

    using Buffer = UninitializedArray<uint32_t, CAPACITY>;
    using Type = Buffer::value_type;
    Buffer buffer;

    new (&buffer[0]) Type(INITIAL_VALUE);
    new (&buffer[1]) Type(INITIAL_VALUE + 1);
    new (&buffer[2]) Type(INITIAL_VALUE + 2);

    uint32_t count{0};
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) reuse of buffer to test const methods
    for (const auto& e : *const_cast<const decltype(buffer)*>(&buffer))
    {
        EXPECT_EQ(e, INITIAL_VALUE + count);
        ++count;
    }
    EXPECT_EQ(count, CAPACITY);
}

TEST(UninitializedArrayTest, UninitializedArrayDoesNotInitializeOrDestroyElements)
{
    ::testing::Test::RecordProperty("TEST_ID", "60334385-60e8-49bc-b297-61f5a5a3b175");
    constexpr uint64_t CAPACITY{15};
    Integer::ctor = 0;
    Integer::dtor = 0;

    {
        UninitializedArray<Integer, CAPACITY> buffer{};
        EXPECT_EQ(Integer::ctor, 0);

        for (uint64_t i{0}; i < CAPACITY; ++i)
        {
            new (&buffer[i]) Integer(51);
        }
        EXPECT_EQ(Integer::ctor, CAPACITY);
        EXPECT_EQ(Integer::dtor, 0);
    }
    EXPECT_EQ(Integer::dtor, 0);
}

TYPED_TEST(UninitializedArrayTest, SizeOfUninitializedArrayEqualsCStyleArray)
{
    ::testing::Test::RecordProperty("TEST_ID", "e1c7ddec-b883-4eee-a4a4-a8dfbcaaec6d");
    using Buffer = typename TestFixture::Buffer;
    if (std::is_same<Buffer, UninitializedArray<Integer, 10>>::value
        || std::is_same<Buffer, UninitializedArray<Integer, 10, iox::ZeroedBuffer>>::value)
    {
        // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays) : needed for test purpose
        Integer testArray[10];
        EXPECT_EQ(sizeof(this->buffer), sizeof(testArray));
    }
    else
    {
        // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays) : needed for test purpose
        int testArray[10];
        EXPECT_EQ(sizeof(this->buffer), sizeof(testArray));
    }
}
} // namespace
