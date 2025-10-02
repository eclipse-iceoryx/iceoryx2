// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#include "iox2/container/detail/raw_byte_storage.hpp"

#include "testing/observable.hpp"
#include "testing/test_utils.hpp"

#include "gtest/gtest.h"

namespace {
using iox2::container::testing::Observable;

struct RawByteStorageFixtureLeak : public iox2::container::testing::DetectLeakedObservablesFixture { };
struct RawByteStorageFixtureStrict : public iox2::container::testing::VerifyAllObservableInteractionsFixture { };

TEST(RawByteStorage, construction_initializes_size_to_0) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::container::detail::RawByteStorage<char, STORAGE_CAPACITY> const sut;
    ASSERT_EQ(sut.size(), 0);
}

TEST(RawByteStorage, construction_initializes_all_storage_bytes_to_0) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::container::detail::RawByteStorage<char, STORAGE_CAPACITY> sut;
    // NOLINTBEGIN(cppcoreguidelines-pro-bounds-pointer-arithmetic) testing
    ASSERT_EQ(sut.pointer_from_index(0)[0], '\0');
    ASSERT_EQ(sut.pointer_from_index(0)[1], '\0');
    ASSERT_EQ(sut.pointer_from_index(0)[2], '\0');
    ASSERT_EQ(sut.pointer_from_index(0)[3], '\0');
    ASSERT_EQ(sut.pointer_from_index(0)[4], '\0');
    // NOLINTEND(cppcoreguidelines-pro-bounds-pointer-arithmetic)
}

TEST(RawByteStorage, storage_is_aligned_suitably_for_type) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    auto determine_alignment = [](void* ptr) -> uint64_t {
        if (!ptr) {
            return 0;
        }
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast) testing
        auto pti = reinterpret_cast<std::uintptr_t>(ptr);
        uint64_t ret = ((pti & 0x01U) == 0U) ? 1 : 0;
        while ((pti & 0x01U) == 0U) {
            ret *= 2;
            pti >>= 1U;
        }
        return ret;
    };
    {
        iox2::container::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), alignof(int32_t));
    }
    {
        iox2::container::detail::RawByteStorage<int64_t, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), alignof(int64_t));
    }
    {
        constexpr size_t const EXTENDED_ALIGNMENT = 128;
        struct alignas(EXTENDED_ALIGNMENT) Overaligned { };
        iox2::container::detail::RawByteStorage<Overaligned, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), 128);
    }
    {
        constexpr size_t const PAGE_ALIGNMENT = 4096;
        struct alignas(PAGE_ALIGNMENT) PageAligned { };
        iox2::container::detail::RawByteStorage<PageAligned, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), 4096);
    }
}

TEST(RawByteStorage, emplace_back_inserts_a_new_element_at_the_back) {
    constexpr uint64_t const STORAGE_CAPACITY = 3;
    iox2::container::detail::RawByteStorage<int64_t, STORAGE_CAPACITY> sut;
    int64_t const test_value = 12345678;
    sut.emplace_back(test_value);
    ASSERT_EQ(sut.size(), 1);
    ASSERT_EQ(*sut.pointer_from_index(0), test_value);
    int64_t const another_test_value = 987654321;
    sut.emplace_back(another_test_value);
    ASSERT_EQ(sut.size(), 2);
    ASSERT_EQ(*sut.pointer_from_index(0), test_value);
    ASSERT_EQ(*sut.pointer_from_index(1), another_test_value);
    int64_t const third_test_value = -10;
    sut.emplace_back(third_test_value);
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(*sut.pointer_from_index(0), test_value);
    ASSERT_EQ(*sut.pointer_from_index(1), another_test_value);
    ASSERT_EQ(*sut.pointer_from_index(2), third_test_value);
}

TEST_F(RawByteStorageFixtureLeak, emplace_back_forwards_arguments_for_initialization) {
    constexpr uint64_t const STORAGE_CAPACITY = 3;
    int32_t const tracking_id1 = 42;
    int32_t const tracking_id2 = 99;
    Observable test_object;
    test_object.id = tracking_id1;
    {
        iox2::container::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        sut.emplace_back(test_object);
        EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
        EXPECT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(sut.size(), 1);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        test_object.id = tracking_id2;
        sut.emplace_back(std::move(test_object));
        EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
        EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(sut.size(), 2);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
    }
    EXPECT_EQ(Observable::s_counter.was_destructed, 2);
}

TEST_F(RawByteStorageFixtureStrict, copy_constructor_copies_all_elements) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::container::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::container::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut { obj };
        iox2::container::testing::opaque_use(&sut);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 3);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 3);
    ASSERT_EQ(obj.size(), 3);
    EXPECT_EQ(obj.pointer_from_index(0)->id, tracking_id1);
    EXPECT_EQ(obj.pointer_from_index(1)->id, tracking_id2);
    EXPECT_EQ(obj.pointer_from_index(2)->id, tracking_id3);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_copy_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, copy_constructor_to_larger_target_capacity_copies_all_elements) {
    constexpr uint64_t const SOURCE_CAPACITY = 4;
    constexpr uint64_t const TARGET_CAPACITY = 5;
    iox2::container::detail::RawByteStorage<Observable, SOURCE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::container::detail::RawByteStorage<Observable, TARGET_CAPACITY> sut { obj };
        iox2::container::testing::opaque_use(&sut);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 3);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 3);
    ASSERT_EQ(obj.size(), 3);
    EXPECT_EQ(obj.pointer_from_index(0)->id, tracking_id1);
    EXPECT_EQ(obj.pointer_from_index(1)->id, tracking_id2);
    EXPECT_EQ(obj.pointer_from_index(2)->id, tracking_id3);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_copy_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_constructor_moves_all_elements) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::container::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::container::detail::RawByteStorage<Observable, STORAGE_CAPACITY> const sut { std::move(obj) };
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 3);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    // NOLINTBEGIN(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move) testing
    ASSERT_EQ(Observable::s_counter.was_destructed, 3);
    ASSERT_EQ(obj.size(), 3);
    int32_t const tracking_id_moved_from = 0;
    EXPECT_EQ(obj.pointer_from_index(0)->id, tracking_id_moved_from);
    EXPECT_EQ(obj.pointer_from_index(1)->id, tracking_id_moved_from);
    EXPECT_EQ(obj.pointer_from_index(2)->id, tracking_id_moved_from);
    // NOLINTEND(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move) testing
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_move_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_constructor_to_larger_capacity_moves_all_elements) {
    constexpr uint64_t const SOURCE_CAPACITY = 4;
    constexpr uint64_t const TARGET_CAPACITY = 5;
    iox2::container::detail::RawByteStorage<Observable, SOURCE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::container::detail::RawByteStorage<Observable, TARGET_CAPACITY> const sut { std::move(obj) };
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 3);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    // NOLINTBEGIN(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move) testing
    ASSERT_EQ(Observable::s_counter.was_destructed, 3);
    ASSERT_EQ(obj.size(), 3);
    int32_t const tracking_id_moved_from = 0;
    EXPECT_EQ(obj.pointer_from_index(0)->id, tracking_id_moved_from);
    EXPECT_EQ(obj.pointer_from_index(1)->id, tracking_id_moved_from);
    EXPECT_EQ(obj.pointer_from_index(2)->id, tracking_id_moved_from);
    // NOLINTEND(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move) testing
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_move_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity) testing
TEST(RawByteStorage, destructor_destructs_elements_from_back_to_front) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    // NOLINTBEGIN(misc-non-private-member-variables-in-classes,cppcoreguidelines-special-member-functions,hicpp-special-member-functions) testing
    class DestructionOrderTracker {
      public:
        int32_t create_order_index = 1;
        size_t destruction_order_index = 0;
        // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) testing
        int32_t destruction_order_array[STORAGE_CAPACITY] = {};

        struct TrackObject {
            int32_t i;
            DestructionOrderTracker* tracker;
            explicit TrackObject(int32_t obj_id, DestructionOrderTracker* tracker)
                : i(obj_id)
                , tracker(tracker) {
            }
            TrackObject(TrackObject&& rhs) noexcept
                : i(std::exchange(rhs.i, 0))
                , tracker(std::exchange(rhs.tracker, nullptr)) {
            }
            ~TrackObject() {
                if (tracker != nullptr) {
                    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index) testing
                    tracker->destruction_order_array[tracker->destruction_order_index] = i;
                    ++tracker->destruction_order_index;
                }
            }
        };
        auto new_object() -> TrackObject {
            return TrackObject(create_order_index++, this);
        }
    } tracker;
    // NOLINTEND(misc-non-private-member-variables-in-classes,cppcoreguidelines-special-member-functions,hicpp-special-member-functions)
    {
        iox2::container::detail::RawByteStorage<DestructionOrderTracker::TrackObject, STORAGE_CAPACITY> sut;
        sut.emplace_back(tracker.new_object());
        sut.emplace_back(tracker.new_object());
        sut.emplace_back(tracker.new_object());
        ASSERT_EQ(sut.pointer_from_index(0)->i, 1);
        ASSERT_EQ(sut.pointer_from_index(1)->i, 2);
        ASSERT_EQ(sut.pointer_from_index(2)->i, 3);
    }
    ASSERT_EQ(tracker.destruction_order_index, 3);
    ASSERT_EQ(tracker.destruction_order_array[0], 3);
    ASSERT_EQ(tracker.destruction_order_array[1], 2);
    ASSERT_EQ(tracker.destruction_order_array[2], 1);
}

} // namespace
