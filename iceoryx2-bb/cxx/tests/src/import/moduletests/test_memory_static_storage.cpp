// Copyright (c) 2020 - 2021 by Apex.AI Inc. All rights reserved.
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

#include "iox/detail/static_storage.hpp"
#include "test.hpp"

#include <iostream>

using namespace ::testing;
using namespace iox;


// note we cannot enforce size and alignment at the same time,
// only minimum size and alignment
// i.e. using Bytes = alignas(Align) uint8_t[Size];
// will not be aligned, alignas is ignored) so we have to use a struct

// the actual size will be some multiple of a power of 2 larger than MinSize
// decided by the compiler
template <uint32_t Size, uint32_t Align = 1U>
struct alignas(Align) Bytes
{
    // NOLINTJUSTIFICATION required to provide raw memory in tests
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    uint8_t data[Size] = {};

    void set(uint8_t value)
    {
        memset(&data[0], value, Size);
    }

    bool hasValue(uint8_t value)
    {
        for (uint32_t i = 0; i < Size; ++i)
        {
            // NOLINTJUSTIFICATION verify content of memory
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
            if (data[i] != value)
            {
                return false;
            }
        }
        return true;
    }
};

namespace
{
TEST(static_storage_test, CapacityIsConsistent)
{
    ::testing::Test::RecordProperty("TEST_ID", "0b533f98-4a17-4480-bf53-9c44256a0d3c");
    constexpr uint64_t CAPACITY = 16;
    EXPECT_EQ(static_storage<CAPACITY>::capacity(), CAPACITY);
}

TEST(static_storage_test, IsAllocatableWithoutAlignmentRestriction)
{
    ::testing::Test::RecordProperty("TEST_ID", "62ad55d1-1505-43ec-9af0-3a73333515aa");
    using Data = Bytes<16, 1>;
    EXPECT_EQ(sizeof(Data), 16);
    EXPECT_EQ(alignof(Data), 1);
    EXPECT_TRUE(static_storage<16>::is_allocatable<Data>());
}

TEST(static_storage_test, IsNotAllocatableWithoutAlignmentRestriction)
{
    ::testing::Test::RecordProperty("TEST_ID", "000f64bc-2f02-4f58-97d2-112464b09e5b");
    using Data = Bytes<16, 1>;
    EXPECT_FALSE(static_storage<15>::is_allocatable<Data>());
}

TEST(static_storage_test, IsAllocatableWithAlignmentRestriction)
{
    ::testing::Test::RecordProperty("TEST_ID", "baba3141-3c3f-42f3-982c-1763410becb7");
    using Data = Bytes<16, 4>;
    EXPECT_EQ(sizeof(Data), 16);
    EXPECT_EQ(alignof(Data), 4);
    EXPECT_TRUE(static_storage<19>::is_allocatable<Data>());
}

TEST(static_storage_test, IsNotAllocatableWithAlignmentRestriction)
{
    ::testing::Test::RecordProperty("TEST_ID", "ca960e9a-a6d8-4fc4-b57a-cb2c665adffd");
    using Data = Bytes<16, 4>;
    EXPECT_FALSE(static_storage<18>::is_allocatable<Data>());
}

TEST(static_storage_test, IsAllocatableWithDifferentAlignment)
{
    ::testing::Test::RecordProperty("TEST_ID", "66341003-4ec2-4990-8321-7df49eb5e79f");
    using Data = Bytes<16, 4>;
    const bool result = static_storage<18, 2>::is_allocatable<Data>();
    EXPECT_TRUE(result);
}

TEST(static_storage_test, IsNotAllocatableWithDifferentAlignment)
{
    ::testing::Test::RecordProperty("TEST_ID", "17e4b597-fe98-4b76-93df-c792d61453ff");
    using Data = Bytes<16, 4>;
    const bool result = static_storage<17, 2>::is_allocatable<Data>();
    EXPECT_FALSE(result);
}

TEST(static_storage_test, AllocateSucceedsIfSizeIsSufficient)
{
    ::testing::Test::RecordProperty("TEST_ID", "7e1bc0e1-a2f3-46d5-ba6a-5b05dcf50df0");
    static_storage<18, 2> sut;
    EXPECT_NE(sut.allocate(16, 4), nullptr);
}

TEST(static_storage_test, AllocateFailsIfSizeIsInsufficient)
{
    ::testing::Test::RecordProperty("TEST_ID", "164dde97-c0a4-44d9-b6cc-3fd57611c15d");
    static_storage<17, 2> sut;
    EXPECT_NE(sut.allocate(16, 4), nullptr);
}

TEST(static_storage_test, TypedAllocateSucceedsIfSizeIsSufficient)
{
    ::testing::Test::RecordProperty("TEST_ID", "7fcc223e-9247-4446-a55c-bede75b15257");
    using Data = Bytes<16, 4>;
    static_storage<18, 2> sut;
    EXPECT_NE(sut.allocate<Data>(), nullptr);
    // when size of storage is insufficent it will not compile and
    // therefore cannot be tested
}

TEST(static_storage_test, DoubleAllocateFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "aa935996-284c-4c7f-a738-04cf66017d9c");
    static_storage<18, 2> sut;
    sut.allocate(16, 4);
    EXPECT_EQ(sut.allocate(16, 4), nullptr);
}

TEST(static_storage_test, DoubleTypedAllocateFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "04847320-17ea-4532-8b1f-3fb9a2024998");
    using Data = Bytes<16, 4>;
    static_storage<18, 2> sut;
    sut.allocate<Data>();
    EXPECT_EQ(sut.allocate<Data>(), nullptr);
}

TEST(static_storage_test, AllocateAfterDeallocateSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "75904d93-d2dd-4df4-88e0-0e35925ec51c");
    static_storage<18, 2> sut;
    sut.allocate(16, 4);
    sut.deallocate();
    EXPECT_NE(sut.allocate(16, 4), nullptr);
}

TEST(static_storage_test, TypedAllocateAfterDeallocateSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "b0287c9b-389e-4e8c-974f-b33166c29e16");
    using Data = Bytes<16, 4>;
    static_storage<18, 2> sut;
    sut.allocate<Data>();
    sut.deallocate();
    EXPECT_NE(sut.allocate<Data>(), nullptr);
}

TEST(static_storage_test, ClearSetsStorageBytesToZeroIfThereIsNoObjectStored)
{
    ::testing::Test::RecordProperty("TEST_ID", "debd1562-2b68-485b-a5df-d38ecf50e3ef");
    using Data = Bytes<16, 4>;
    static_storage<18, 2> sut;
    auto* data = sut.allocate<Data>();
    ASSERT_NE(data, nullptr);
    data->set(37);
    EXPECT_TRUE(data->hasValue(37));

    sut.deallocate();
    EXPECT_TRUE(sut.clear());
    EXPECT_TRUE(data->hasValue(0));
}

TEST(static_storage_test, ClearHasNoEffectIfThereIsAnObjectStored)
{
    ::testing::Test::RecordProperty("TEST_ID", "8882ef4d-92df-4370-9f84-5b6ead3d6d2c");
    using Data = Bytes<16, 4>;
    static_storage<18, 2> sut;
    auto* data = sut.allocate<Data>();
    ASSERT_NE(data, nullptr);
    data->set(37);
    EXPECT_TRUE(data->hasValue(37));

    EXPECT_FALSE(sut.clear());
    EXPECT_TRUE(data->hasValue(37));
}

TEST(static_storage_test, AllocationIsAligned)
{
    ::testing::Test::RecordProperty("TEST_ID", "645c0194-7aea-4f9c-b379-212fbcaa05f7");
    static_storage<17, 2> sut;
    // NOLINTJUSTIFICATION required for testing
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    auto p = reinterpret_cast<uintptr_t>(sut.allocate(16, 4));
    EXPECT_EQ(p % 4, 0);
}

TEST(static_storage_test, TypedAllocationIsAligned)
{
    ::testing::Test::RecordProperty("TEST_ID", "bb990529-2721-4db8-8b17-02719021210e");
    using Data = Bytes<4, 8>;
    static_storage<17, 2> sut;
    // NOLINTJUSTIFICATION required for testing
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    auto p = reinterpret_cast<uintptr_t>(sut.allocate<Data>());
    EXPECT_EQ(p % 8, 0);
}

TEST(static_storage_test, AllocationSizeReturnsSizeIfTypeIsAlignedWithStorage)
{
    ::testing::Test::RecordProperty("TEST_ID", "22106a23-2bd4-40ef-9a1c-112f91f254da");
    constexpr uint64_t typeAlign = 2;
    constexpr uint64_t storageAlign = 2 * typeAlign;
    using Data = Bytes<4, typeAlign>;

    constexpr auto size = static_storage<17, storageAlign>::allocation_size<Data>();
    EXPECT_EQ(size, sizeof(Data));
}

TEST(static_storage_test, AllocationSizeReturnsMoreThanSizeIfTypeIsNotAlignedWithStorage)
{
    ::testing::Test::RecordProperty("TEST_ID", "76f41902-fa90-481b-bca8-2a897b3ce7c7");
    constexpr uint64_t typeAlign = 16;
    constexpr uint64_t storageAlign = 4;
    using Data = Bytes<4, typeAlign>;

    constexpr auto size = static_storage<17, storageAlign>::allocation_size<Data>();
    EXPECT_EQ(size, sizeof(Data) + typeAlign - storageAlign);
}

} // namespace
