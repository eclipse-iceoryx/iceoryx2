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

#include "iox2/bb/detail/raw_byte_storage.hpp"

#include "testing/observable.hpp"
#include "testing/test_utils.hpp"

#include "gtest/gtest.h"

namespace {
using iox2::bb::testing::Observable;

struct RawByteStorageFixtureLeak : public iox2::bb::testing::DetectLeakedObservablesFixture { };
struct RawByteStorageFixtureStrict : public iox2::bb::testing::VerifyAllObservableInteractionsFixture { };

TEST(RawByteStorage, construction_initializes_size_to_0) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<char, STORAGE_CAPACITY> const sut;
    ASSERT_EQ(sut.size(), 0);
}

TEST(RawByteStorage, construction_initializes_all_storage_bytes_to_0) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<char, STORAGE_CAPACITY> sut;
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
        iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), alignof(int32_t));
    }
    {
        iox2::bb::detail::RawByteStorage<int64_t, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), alignof(int64_t));
    }
    {
        constexpr size_t const EXTENDED_ALIGNMENT = 128;
        struct alignas(EXTENDED_ALIGNMENT) Overaligned { };
        iox2::bb::detail::RawByteStorage<Overaligned, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), 128);
    }
    {
        constexpr size_t const PAGE_ALIGNMENT = 4096;
        struct alignas(PAGE_ALIGNMENT) PageAligned { };
        iox2::bb::detail::RawByteStorage<PageAligned, STORAGE_CAPACITY> sut;
        ASSERT_GE(determine_alignment(sut.pointer_from_index(0)), 4096);
    }
}

TEST(RawByteStorage, emplace_back_inserts_a_new_element_at_the_back) {
    constexpr uint64_t const STORAGE_CAPACITY = 3;
    iox2::bb::detail::RawByteStorage<int64_t, STORAGE_CAPACITY> sut;
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
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
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
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut { obj };
        iox2::bb::testing::opaque_use(&sut);
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

TEST_F(RawByteStorageFixtureStrict, copy_assignment_copies_all_elements_target_empty) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        iox2::bb::testing::opaque_use(&sut);
        sut = obj;
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
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
    expected_count().was_copy_assigned = 0;
    expected_count().was_copy_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, copy_assignment_copies_all_elements_target_partially_filled) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        sut.emplace_back();
        sut.emplace_back();
        iox2::bb::testing::opaque_use(&sut);
        sut = obj;
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 2);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
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
    expected_count().was_initialized = 5;
    expected_count().was_copy_assigned = 2;
    expected_count().was_copy_constructed = 1;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, copy_assignment_copies_all_elements_target_filled) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        iox2::bb::testing::opaque_use(&sut);
        sut = obj;
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 3);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
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
    expected_count().was_initialized = 6;
    expected_count().was_copy_assigned = 3;
    expected_count().was_copy_constructed = 0;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, copy_assignment_copies_all_elements_target_bigger) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        iox2::bb::testing::opaque_use(&sut);
        sut = obj;
        ASSERT_EQ(Observable::s_counter.was_copy_assigned, 3);
        ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 2);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 5);
    ASSERT_EQ(obj.size(), 3);
    EXPECT_EQ(obj.pointer_from_index(0)->id, tracking_id1);
    EXPECT_EQ(obj.pointer_from_index(1)->id, tracking_id2);
    EXPECT_EQ(obj.pointer_from_index(2)->id, tracking_id3);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 8;
    expected_count().was_copy_assigned = 3;
    expected_count().was_copy_constructed = 0;
    expected_count().was_destructed = 8;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, copy_assignment_returns_reference_to_this) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        iox2::bb::testing::opaque_use(&sut);
        ASSERT_EQ(&(sut = obj), &sut);
    }
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_copy_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, copy_assignment_self_assignment_is_noop) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    sut.emplace_back(tracking_id1);
    sut.emplace_back(tracking_id2);
    sut.emplace_back(tracking_id3);
    auto& alias_to_sut = sut;
    iox2::bb::testing::opaque_use(&alias_to_sut);
    sut = alias_to_sut;
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_destructed = 3;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_assignment_copies_all_elements_target_empty) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        iox2::bb::testing::opaque_use(&sut);
        sut = std::move(obj);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 3);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 3);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_move_assigned = 0;
    expected_count().was_move_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_assignment_copies_all_elements_target_partially_filled) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        sut.emplace_back();
        sut.emplace_back();
        iox2::bb::testing::opaque_use(&sut);
        sut = std::move(obj);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 2);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 1);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 3);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 5;
    expected_count().was_move_assigned = 2;
    expected_count().was_move_constructed = 1;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_assignment_copies_all_elements_target_filled) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        iox2::bb::testing::opaque_use(&sut);
        sut = std::move(obj);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 3);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 3);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 6;
    expected_count().was_move_assigned = 3;
    expected_count().was_move_constructed = 0;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_assignment_copies_all_elements_target_bigger) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        sut.emplace_back();
        iox2::bb::testing::opaque_use(&sut);
        sut = std::move(obj);
        ASSERT_EQ(Observable::s_counter.was_move_assigned, 3);
        ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
        ASSERT_EQ(sut.size(), 3);
        EXPECT_EQ(sut.pointer_from_index(0)->id, tracking_id1);
        EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id2);
        EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id3);
        ASSERT_EQ(Observable::s_counter.was_destructed, 2);
    }
    ASSERT_EQ(Observable::s_counter.was_destructed, 5);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 8;
    expected_count().was_move_assigned = 3;
    expected_count().was_move_constructed = 0;
    expected_count().was_destructed = 8;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_assignment_returns_reference_to_this) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
        iox2::bb::testing::opaque_use(&sut);
        ASSERT_EQ(&(sut = std::move(obj)), &sut);
    }
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_move_constructed = 3;
    expected_count().was_destructed = 6;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, move_assignment_self_assignment_is_noop) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    sut.emplace_back(tracking_id1);
    sut.emplace_back(tracking_id2);
    sut.emplace_back(tracking_id3);
    auto& alias_to_sut = sut;
    iox2::bb::testing::opaque_use(&alias_to_sut);
    sut = std::move(alias_to_sut);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) testing
    expected_count().was_initialized = 3;
    expected_count().was_destructed = 3;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(RawByteStorageFixtureStrict, copy_constructor_to_larger_target_capacity_copies_all_elements) {
    constexpr uint64_t const SOURCE_CAPACITY = 4;
    constexpr uint64_t const TARGET_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<Observable, SOURCE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, TARGET_CAPACITY> sut { obj };
        iox2::bb::testing::opaque_use(&sut);
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
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> const sut { std::move(obj) };
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
    iox2::bb::detail::RawByteStorage<Observable, SOURCE_CAPACITY> obj;
    int32_t const tracking_id1 = 100;
    int32_t const tracking_id2 = 200;
    int32_t const tracking_id3 = 300;
    obj.emplace_back(tracking_id1);
    obj.emplace_back(tracking_id2);
    obj.emplace_back(tracking_id3);
    {
        iox2::bb::detail::RawByteStorage<Observable, TARGET_CAPACITY> const sut { std::move(obj) };
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
        iox2::bb::detail::RawByteStorage<DestructionOrderTracker::TrackObject, STORAGE_CAPACITY> sut;
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

TEST(RawByteStorage, emplace_at_inserts_in_the_middle_of_a_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_at(1, marker_value);
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(2), 2);
}

TEST(RawByteStorage, emplace_at_inserts_at_the_beginning_of_a_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_at(0, marker_value);
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.pointer_from_index(0), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(1), 1);
    EXPECT_EQ(*sut.pointer_from_index(2), 2);
}

TEST(RawByteStorage, emplace_at_inserts_at_the_end_of_a_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_at(2, marker_value);
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), 2);
    EXPECT_EQ(*sut.pointer_from_index(2), marker_value);
}

TEST(RawByteStorage, emplace_at_inserts_into_empty_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_at(0, marker_value);
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(*sut.pointer_from_index(0), marker_value);
}

TEST_F(RawByteStorageFixtureLeak, emplace_at_does_not_copy_objects_for_relocation) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(4);
    sut.emplace_at(1, marker_value);
    ASSERT_EQ(sut.size(), 5);
    EXPECT_EQ(sut.pointer_from_index(0)->id, 1);
    EXPECT_EQ(sut.pointer_from_index(1)->id, marker_value);
    EXPECT_EQ(sut.pointer_from_index(2)->id, 2);
    EXPECT_EQ(sut.pointer_from_index(3)->id, 3);
    EXPECT_EQ(sut.pointer_from_index(4)->id, 4);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    // a range of 4 elements needs to be rotated, leading in the worst case to 4 swaps, consisting of up to 3 moves each
    ASSERT_LE(Observable::s_counter.was_move_assigned + Observable::s_counter.was_move_constructed, 12);
}

TEST(RawByteStorage, insert_at_inserts_multiple_elements_in_the_middle_of_a_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    uint64_t const element_count = 5;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.insert_at(1, element_count, marker_value);
    ASSERT_EQ(sut.size(), 7);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(2), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(3), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(4), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(5), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(6), 2);
}

TEST(RawByteStorage, insert_at_inserts_multiple_elements_at_the_beginning_of_a_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    uint64_t const element_count = 5;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.insert_at(0, element_count, marker_value);
    ASSERT_EQ(sut.size(), 7);
    EXPECT_EQ(*sut.pointer_from_index(0), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(1), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(2), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(3), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(4), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(5), 1);
    EXPECT_EQ(*sut.pointer_from_index(6), 2);
}

TEST(RawByteStorage, insert_at_inserts_multiple_elements_at_the_end_of_a_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    uint64_t const element_count = 5;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.insert_at(2, element_count, marker_value);
    ASSERT_EQ(sut.size(), 7);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), 2);
    EXPECT_EQ(*sut.pointer_from_index(2), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(3), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(4), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(5), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(6), marker_value);
}

TEST(RawByteStorage, insert_at_inserts_multiple_elements_into_empty_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    uint64_t const element_count = 5;
    sut.insert_at(0, element_count, marker_value);
    ASSERT_EQ(sut.size(), 5);
    EXPECT_EQ(*sut.pointer_from_index(0), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(1), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(2), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(3), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(4), marker_value);
}

TEST(RawByteStorage, insert_at_single_element) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(4);
    sut.insert_at(1, 1, marker_value);
    ASSERT_EQ(sut.size(), 5);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), marker_value);
    EXPECT_EQ(*sut.pointer_from_index(2), 2);
    EXPECT_EQ(*sut.pointer_from_index(3), 3);
    EXPECT_EQ(*sut.pointer_from_index(4), 4);
}

TEST(RawByteStorage, insert_at_with_zero_elements_does_nothing) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(4);
    sut.insert_at(1, 0, marker_value);
    ASSERT_EQ(sut.size(), 4);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), 2);
    EXPECT_EQ(*sut.pointer_from_index(2), 3);
    EXPECT_EQ(*sut.pointer_from_index(3), 4);
}

TEST_F(RawByteStorageFixtureLeak, insert_at_does_not_copy_elements_for_relocation) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
    int32_t const tracking_id = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(4);
    sut.insert_at(1, 4, Observable { tracking_id });
    ASSERT_EQ(sut.size(), 8);
    EXPECT_EQ(sut.pointer_from_index(0)->id, 1);
    EXPECT_EQ(sut.pointer_from_index(1)->id, tracking_id);
    EXPECT_EQ(sut.pointer_from_index(2)->id, tracking_id);
    EXPECT_EQ(sut.pointer_from_index(3)->id, tracking_id);
    EXPECT_EQ(sut.pointer_from_index(4)->id, tracking_id);
    EXPECT_EQ(sut.pointer_from_index(5)->id, 2);
    EXPECT_EQ(sut.pointer_from_index(6)->id, 3);
    EXPECT_EQ(sut.pointer_from_index(7)->id, 4);
    // copy construction is used for exactly the inserted elements
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 4);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    // a range of 7 elements needs to be moved, leading in the worst case to 7 swaps, consisting of up to 3 moves each
    ASSERT_LE(Observable::s_counter.was_move_assigned + Observable::s_counter.was_move_constructed, 21);
}

TEST(RawByteStorage, erase_at_erases_single_element) {
    constexpr uint64_t const STORAGE_CAPACITY = 5;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(4);
    // middle
    sut.erase_at(1);
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), 3);
    EXPECT_EQ(*sut.pointer_from_index(2), 4);
    // beginning
    sut.erase_at(0);
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(*sut.pointer_from_index(0), 3);
    EXPECT_EQ(*sut.pointer_from_index(1), 4);
    // end
    sut.erase_at(1);
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(*sut.pointer_from_index(0), 3);
    // empty
    sut.erase_at(0);
    ASSERT_EQ(sut.size(), 0);
}

TEST_F(RawByteStorageFixtureLeak, erase_at_does_not_copy_elements_for_relocation) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
    int32_t const tracking_id = 99;
    sut.emplace_back(1);
    sut.emplace_back(tracking_id);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(4);
    sut.erase_at(1);
    ASSERT_EQ(sut.size(), 4);
    EXPECT_EQ(sut.pointer_from_index(0)->id, 1);
    EXPECT_EQ(sut.pointer_from_index(1)->id, 2);
    EXPECT_EQ(sut.pointer_from_index(2)->id, 3);
    EXPECT_EQ(sut.pointer_from_index(3)->id, 4);
    // copy construction is used for exactly the inserted elements
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    // a range of 3 elements needs to be moved
    ASSERT_EQ(Observable::s_counter.was_move_assigned, 3);
    ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
}

TEST(RawByteStorage, erase_at_erase_range_from_middle) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(2);
    sut.emplace_back(3);
    uint64_t const range_begin = 1;
    uint64_t const range_end = 6;
    sut.erase_at(range_begin, range_end);
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), 2);
    EXPECT_EQ(*sut.pointer_from_index(2), 3);
}

TEST(RawByteStorage, erase_at_erase_range_from_front) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_back(3);
    uint64_t const range_begin = 0;
    uint64_t const range_end = 5;
    sut.erase_at(range_begin, range_end);
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), 2);
    EXPECT_EQ(*sut.pointer_from_index(2), 3);
}

TEST(RawByteStorage, erase_at_erase_range_from_back) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(1);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    uint64_t const range_begin = 3;
    uint64_t const range_end = 8;
    sut.erase_at(range_begin, range_end);
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.pointer_from_index(0), 1);
    EXPECT_EQ(*sut.pointer_from_index(1), 2);
    EXPECT_EQ(*sut.pointer_from_index(2), 3);
}

TEST(RawByteStorage, erase_at_erase_whole_range) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<int32_t, STORAGE_CAPACITY> sut;
    int32_t const marker_value = 99;
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    sut.emplace_back(marker_value);
    uint64_t const range_begin = 0;
    uint64_t const range_end = 5;
    sut.erase_at(range_begin, range_end);
    ASSERT_EQ(sut.size(), 0);
}

TEST_F(RawByteStorageFixtureLeak, erase_at_range_does_not_copy_elements_for_relocation) {
    constexpr uint64_t const STORAGE_CAPACITY = 10;
    iox2::bb::detail::RawByteStorage<Observable, STORAGE_CAPACITY> sut;
    int32_t const tracking_id = 99;
    sut.emplace_back(1);
    sut.emplace_back(tracking_id);
    sut.emplace_back(tracking_id);
    sut.emplace_back(tracking_id);
    sut.emplace_back(tracking_id);
    sut.emplace_back(2);
    sut.emplace_back(3);
    sut.emplace_back(4);
    uint64_t const range_begin = 1;
    uint64_t const range_end = 5;
    sut.erase_at(range_begin, range_end);
    ASSERT_EQ(sut.size(), 4);
    EXPECT_EQ(sut.pointer_from_index(0)->id, 1);
    EXPECT_EQ(sut.pointer_from_index(1)->id, 2);
    EXPECT_EQ(sut.pointer_from_index(2)->id, 3);
    EXPECT_EQ(sut.pointer_from_index(3)->id, 4);
    // copy construction is used for exactly the inserted elements
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    // a range of 3 elements needs to be moved
    ASSERT_EQ(Observable::s_counter.was_move_assigned, 3);
    ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
}

} // namespace
