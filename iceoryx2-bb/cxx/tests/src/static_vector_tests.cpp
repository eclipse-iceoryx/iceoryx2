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

#include "iox2/bb/static_vector.hpp"

#include "testing/observable.hpp"
#include "testing/test_utils.hpp"

#include "gtest/gtest.h"

#include <sstream>
#include <string>

namespace {
using iox2::bb::testing::Observable;

class StaticVectorFixture : public iox2::bb::testing::VerifyAllObservableInteractionsFixture { };

// NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
constexpr size_t const G_TEST_ARRAY_SIZE = 5;
// NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays)
int32_t const G_TEST_ARRAY[G_TEST_ARRAY_SIZE] = { 4, 9, 77, 32, -5 };


static_assert(std::is_standard_layout<iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>>::value,
              "StaticVector must be standard layout");

TEST(StaticVector, default_constructor_initializes_to_empty) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut;
    ASSERT_TRUE(sut.empty());
}

TEST_F(StaticVectorFixture, default_constructor_does_not_construct_any_objects) {
    iox2::bb::StaticVector<Observable, G_TEST_ARRAY_SIZE> const sut;
    ASSERT_TRUE(sut.empty());
}

TEST(StaticVector, copy_constructor_copies_vector_contents) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> src_vec;
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_TRUE(sut.empty());
        ASSERT_EQ(sut.size(), 0);
    }
    ASSERT_TRUE(src_vec.try_emplace_back(1));
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(sut.size(), 1);
        EXPECT_EQ(sut.unchecked_access()[0], 1);
    }
    ASSERT_TRUE(src_vec.try_emplace_back(2));
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(sut.size(), 2);
        EXPECT_EQ(sut.unchecked_access()[0], 1);
        EXPECT_EQ(sut.unchecked_access()[1], 2);
    }
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[0]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[1]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[3]));
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
        EXPECT_EQ(sut.unchecked_access()[0], 1);
        EXPECT_EQ(sut.unchecked_access()[1], 2);
        EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[0]);
        EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[1]);
        EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[3]);
    }
}

TEST(StaticVector, copy_constructor_copies_vector_contents_to_larger_capacity) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> src_vec;
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_TRUE(sut.empty());
        ASSERT_EQ(sut.size(), 0);
    }
    ASSERT_TRUE(src_vec.try_emplace_back(1));
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(sut.size(), 1);
        EXPECT_EQ(sut.unchecked_access()[0], 1);
    }
    ASSERT_TRUE(src_vec.try_emplace_back(2));
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(sut.size(), 2);
        EXPECT_EQ(sut.unchecked_access()[0], 1);
        EXPECT_EQ(sut.unchecked_access()[1], 2);
    }
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[0]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[1]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[3]));
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(src_vec);
        iox2::bb::testing::opaque_use(sut);
        ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
        EXPECT_EQ(sut.unchecked_access()[0], 1);
        EXPECT_EQ(sut.unchecked_access()[1], 2);
        EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[0]);
        EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[1]);
        EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[3]);
    }
}

TEST(StaticVector, copy_assignment_assigns_vector_contents) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> src_vec;
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut;
    ASSERT_TRUE(sut.empty());
    sut = src_vec;
    ASSERT_TRUE(sut.empty());
    ASSERT_EQ(sut.size(), 0);
    ASSERT_TRUE(src_vec.try_emplace_back(1));
    sut = src_vec;
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_TRUE(src_vec.try_emplace_back(2));
    sut = src_vec;
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[0]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[1]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[3]));
    sut = src_vec;
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[3]);
    src_vec.clear();
    sut = src_vec;
    ASSERT_EQ(sut.size(), 0);
}

TEST(StaticVector, copy_assignment_returns_reference_to_self) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> src_vec;
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut;
    EXPECT_EQ(&(sut = src_vec), &sut);
    ASSERT_TRUE(src_vec.try_push_back(1));
    ASSERT_TRUE(src_vec.try_push_back(2));
    ASSERT_TRUE(src_vec.try_push_back(3));
    EXPECT_EQ(&(sut = src_vec), &sut);
}

TEST(StaticVector, copy_assignment_self_assignment) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    auto const& reference_to_self = sut;
    iox2::bb::testing::opaque_use(&reference_to_self);
    sut = reference_to_self;
    ASSERT_TRUE(!sut.empty());
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
}

TEST(StaticVector, move_assignment_assigns_vector_contents) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> src_vec;
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut;
    ASSERT_TRUE(sut.empty());
    sut = std::move(src_vec);
    src_vec = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> {};
    ASSERT_TRUE(sut.empty());
    ASSERT_EQ(sut.size(), 0);
    ASSERT_TRUE(src_vec.try_emplace_back(1));
    sut = std::move(src_vec);
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    src_vec = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> {};
    ASSERT_TRUE(src_vec.try_emplace_back(1));
    ASSERT_TRUE(src_vec.try_emplace_back(2));
    sut = std::move(src_vec);
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
    src_vec = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> {};
    ASSERT_TRUE(src_vec.try_emplace_back(1));
    ASSERT_TRUE(src_vec.try_emplace_back(2));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[0]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[1]));
    ASSERT_TRUE(src_vec.try_emplace_back(G_TEST_ARRAY[3]));
    sut = std::move(src_vec);
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[3]);
    src_vec = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> {};
    sut = std::move(src_vec);
    ASSERT_EQ(sut.size(), 0);
}

TEST(StaticVector, move_assignment_returns_reference_to_self) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> src_vec;
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut;
    EXPECT_EQ(&(sut = std::move(src_vec)), &sut);
    src_vec = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> {};
    ASSERT_TRUE(src_vec.try_push_back(1));
    ASSERT_TRUE(src_vec.try_push_back(2));
    ASSERT_TRUE(src_vec.try_push_back(3));
    EXPECT_EQ(&(sut = std::move(src_vec)), &sut);
}

TEST(StaticVector, array_constructor_copies_array_elements_into_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut(G_TEST_ARRAY);
    ASSERT_TRUE(!sut.empty());
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
}

TEST(StaticVector, array_constructor_leaves_uninitialized_elements_up_to_capacity) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut(G_TEST_ARRAY);
    ASSERT_TRUE(!sut.empty());
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_TRUE(!sut.element_at(G_TEST_ARRAY_SIZE).has_value());
}

TEST_F(StaticVectorFixture, from_value_default_constructs_count_elements) {
    auto const opt_sut = iox2::bb::StaticVector<Observable, 4>::from_value(4);
    ASSERT_TRUE(opt_sut);
    auto const& sut = *opt_sut;
    ASSERT_EQ(sut.size(), 4);
    EXPECT_EQ(sut.unchecked_access()[0].id, 0);
    EXPECT_EQ(sut.unchecked_access()[1].id, 0);
    EXPECT_EQ(sut.unchecked_access()[2].id, 0);
    EXPECT_EQ(sut.unchecked_access()[3].id, 0);
    ASSERT_EQ(Observable::s_counter.was_initialized, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 4);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    // there may be additional moves on compilers that fail to perform rvo
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_initialized = 1;
    expected_count().was_copy_constructed = 4;
    expected_count().was_destructed = 5 + Observable::s_counter.was_move_constructed;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(StaticVectorFixture, from_value_constructs_empty_vector_for_zero_elements) {
    auto const sut = iox2::bb::StaticVector<Observable, 4>::from_value(0);
    ASSERT_TRUE(sut);
    ASSERT_EQ(sut->size(), 0);
    ASSERT_EQ(Observable::s_counter.was_initialized, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    // there may be additional moves on compilers that fail to perform rvo
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_initialized = 1;
    expected_count().was_copy_constructed = 0;
    expected_count().was_destructed = 1 + Observable::s_counter.was_move_constructed;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(StaticVectorFixture, from_value_fails_if_exceeding_capacity) {
    ASSERT_FALSE((iox2::bb::StaticVector<Observable, 4>::from_value(5)));
}

TEST_F(StaticVectorFixture, from_value_constructs_count_copies_of_element) {
    int32_t const tracking_id = 99;
    Observable const obj { tracking_id };
    auto const sut = iox2::bb::StaticVector<Observable, 4>::from_value(4, obj);
    ASSERT_TRUE(sut);
    ASSERT_EQ(sut->size(), 4);
    EXPECT_EQ(sut->unchecked_access()[0].id, tracking_id);
    EXPECT_EQ(sut->unchecked_access()[1].id, tracking_id);
    EXPECT_EQ(sut->unchecked_access()[2].id, tracking_id);
    EXPECT_EQ(sut->unchecked_access()[3].id, tracking_id);
    ASSERT_EQ(Observable::s_counter.was_initialized, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 4);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    // there may be additional moves on compilers that fail to perform rvo
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_initialized = 1;
    expected_count().was_copy_constructed = 4;
    expected_count().was_destructed = 5 + Observable::s_counter.was_move_constructed;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(StaticVectorFixture, from_value_with_object_constructs_empty_vector_for_zero_elements) {
    int32_t const tracking_id = 99;
    Observable const obj { tracking_id };
    auto const sut = iox2::bb::StaticVector<Observable, 4>::from_value(0, obj);
    ASSERT_TRUE(sut);
    ASSERT_EQ(sut->size(), 0);
    ASSERT_EQ(Observable::s_counter.was_initialized, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    // there may be additional moves on compilers that fail to perform rvo
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_initialized = 1;
    expected_count().was_copy_constructed = 0;
    expected_count().was_destructed = 1 + Observable::s_counter.was_move_constructed;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(StaticVectorFixture, from_value_with_static_count_constructs_count_copies_of_element) {
    int32_t const tracking_id = 142;
    Observable const obj { tracking_id };
    auto const sut = iox2::bb::StaticVector<Observable, 4>::from_value<4>(obj);
    ASSERT_EQ(sut.size(), 4);
    EXPECT_EQ(sut.unchecked_access()[0].id, tracking_id);
    EXPECT_EQ(sut.unchecked_access()[1].id, tracking_id);
    EXPECT_EQ(sut.unchecked_access()[2].id, tracking_id);
    EXPECT_EQ(sut.unchecked_access()[3].id, tracking_id);
    ASSERT_EQ(Observable::s_counter.was_initialized, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 4);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    // there may be additional moves on compilers that fail to perform rvo
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_initialized = 1;
    expected_count().was_copy_constructed = 4;
    expected_count().was_destructed = 5 + Observable::s_counter.was_move_constructed;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(StaticVectorFixture, from_value_with_static_count_constructs_one_copy_of_element) {
    int32_t const tracking_id = 147;
    Observable const obj { tracking_id };
    auto const sut = iox2::bb::StaticVector<Observable, 4>::from_value<1>(obj);
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(sut.unchecked_access()[0].id, tracking_id);
    ASSERT_EQ(Observable::s_counter.was_initialized, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    // there may be additional moves on compilers that fail to perform rvo
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_initialized = 1;
    expected_count().was_copy_constructed = 1;
    expected_count().was_destructed = 2 + Observable::s_counter.was_move_constructed;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}

TEST_F(StaticVectorFixture, from_value_with_static_count_constructs_empty_vector_for_zero_elements) {
    int32_t const tracking_id = 159;
    Observable const obj { tracking_id };
    auto const sut = iox2::bb::StaticVector<Observable, 4>::from_value<0>(obj);
    ASSERT_EQ(sut.size(), 0);
    ASSERT_EQ(Observable::s_counter.was_initialized, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    // there may be additional moves on compilers that fail to perform rvo
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_initialized = 1;
    expected_count().was_copy_constructed = 0;
    expected_count().was_destructed = 1 + Observable::s_counter.was_move_constructed;
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
}


TEST_F(StaticVectorFixture, from_value_with_object_fails_if_exceeding_capacity) {
    int32_t const tracking_id = 99;
    Observable const obj { tracking_id };
    ASSERT_FALSE((iox2::bb::StaticVector<Observable, 4>::from_value(5, obj)));
    expected_count().was_initialized = 1;
    expected_count().was_destructed = 1;
}

TEST(StaticVector, from_range_unchecked_constructs_from_range) {
    auto const sut = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_range_unchecked(std::begin(G_TEST_ARRAY),
                                                                                              std::end(G_TEST_ARRAY));
    ASSERT_TRUE(sut);
    ASSERT_EQ(sut->size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(sut->unchecked_access()[0], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut->unchecked_access()[1], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut->unchecked_access()[2], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut->unchecked_access()[3], G_TEST_ARRAY[3]);
    EXPECT_EQ(sut->unchecked_access()[4], G_TEST_ARRAY[4]);
}

TEST(StaticVector, from_range_unchecked_from_empty_range_constructs_empty_vector) {
    auto const sut = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_range_unchecked(std::end(G_TEST_ARRAY),
                                                                                              std::end(G_TEST_ARRAY));
    ASSERT_TRUE(sut);
    ASSERT_TRUE(sut->empty());
}

TEST(StaticVector, from_range_unchecked_fails_if_exceeding_capacity) {
    auto const sut = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE - 1>::from_range_unchecked(
        std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY));
    ASSERT_FALSE(sut);
}

TEST(StaticVector, from_range_unchecked_constructs_from_range_object) {
    auto const sut = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_range_unchecked(G_TEST_ARRAY);
    ASSERT_TRUE(sut);
    ASSERT_EQ(sut->size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(sut->unchecked_access()[0], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut->unchecked_access()[1], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut->unchecked_access()[2], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut->unchecked_access()[3], G_TEST_ARRAY[3]);
    EXPECT_EQ(sut->unchecked_access()[4], G_TEST_ARRAY[4]);
}

TEST(StaticVector, from_range_unchecked_fails_if_range_object_is_exceeding_capacity) {
    auto const sut = iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE - 1>::from_range_unchecked(G_TEST_ARRAY);
    ASSERT_FALSE(sut);
}

TEST(StaticVector, from_initializer_list_construction) {
    auto const sut = iox2::bb::StaticVector<int32_t, 4>::from_initializer_list({ 1, 2, 3, 4 });
    ASSERT_TRUE(sut);
    ASSERT_EQ(sut->size(), 4);
    ASSERT_EQ(sut->unchecked_access()[0], 1);
    ASSERT_EQ(sut->unchecked_access()[1], 2);
    ASSERT_EQ(sut->unchecked_access()[2], 3);
    ASSERT_EQ(sut->unchecked_access()[3], 4);
}

TEST(StaticVector, from_initializer_list_fails_if_exceeding_capacity) {
    ASSERT_FALSE((iox2::bb::StaticVector<int32_t, 3>::from_initializer_list({ 1, 2, 3, 4 })));
    ASSERT_FALSE((iox2::bb::StaticVector<int32_t, 4>::from_initializer_list({ 0, 0, 0, 0, 0 })));
}

TEST(StaticVector, try_push_back_inserts_elements_at_the_back_if_there_is_room) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(G_TEST_ARRAY);
    int32_t const test_value = 99;
    ASSERT_TRUE(sut.try_push_back(test_value));
    EXPECT_EQ(*sut.element_at(G_TEST_ARRAY_SIZE), test_value);
}

TEST(StaticVector, try_push_back_returns_false_if_there_is_no_room) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    int32_t const test_value = 99;
    ASSERT_TRUE(!sut.try_push_back(test_value));
}

TEST_F(StaticVectorFixture, try_push_back_copies_values_into_vector) {
    {
        iox2::bb::StaticVector<Observable, G_TEST_ARRAY_SIZE> sut;
        int32_t const contained_value = 12345;
        {
            Observable const observable_value { contained_value };
            expected_count().was_initialized = 1;
            ASSERT_TRUE(sut.try_push_back(observable_value));
            ASSERT_TRUE(sut.element_at(0).has_value());
            EXPECT_EQ(sut.element_at(0).value().get().id, contained_value);
            EXPECT_EQ(Observable::s_counter.was_initialized, 1);
            EXPECT_EQ(Observable::s_counter.was_destructed, 0);
        }
        EXPECT_EQ(Observable::s_counter.was_destructed, 1);
        expected_count().was_copy_constructed = 1;
        EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
    }
    EXPECT_EQ(Observable::s_counter.was_destructed, 2);
    expected_count().was_destructed = 2;
}

TEST_F(StaticVectorFixture, try_push_back_moves_temporaries_into_vector) {
    {
        iox2::bb::StaticVector<Observable, G_TEST_ARRAY_SIZE> sut;
        int32_t const contained_value = 12345;
        ASSERT_TRUE(sut.try_push_back(Observable { contained_value }));
        ASSERT_TRUE(sut.element_at(0).has_value());
        EXPECT_EQ(sut.element_at(0).value().get().id, contained_value);
        expected_count().was_initialized = 1;
        EXPECT_EQ(Observable::s_counter.was_initialized, 1);
        expected_count().was_move_constructed = 1;
        EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
        EXPECT_EQ(Observable::s_counter.was_destructed, 1);
    }
    EXPECT_EQ(Observable::s_counter.was_destructed, 2);
    expected_count().was_destructed = 2;
}

TEST_F(StaticVectorFixture, try_push_back_fails_for_temporaries_if_vector_is_full) {
    constexpr size_t const SMALL_VECTOR_CAPACITY = 1;
    iox2::bb::StaticVector<Observable, SMALL_VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_push_back(Observable {}));
    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
    ASSERT_FALSE(sut.try_push_back(Observable {}));
    EXPECT_EQ(Observable::s_counter.was_initialized, 2);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
    expected_count().was_initialized = 2;
    expected_count().was_move_constructed = 1;
    expected_count().was_destructed = 3;
}

TEST_F(StaticVectorFixture, try_emplace_back_constructs_in_place_at_back_of_vector) {
    constexpr size_t const VECTOR_CAPACITY = 5;
    iox2::bb::StaticVector<Observable, VECTOR_CAPACITY> sut;
    int32_t const tracking_id = 99;
    ASSERT_TRUE(sut.try_emplace_back());
    ASSERT_TRUE(sut.try_emplace_back(tracking_id));
    EXPECT_EQ(Observable::s_counter.was_initialized, 2);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0].id, 0);
    EXPECT_EQ(sut.unchecked_access()[1].id, tracking_id);
    expected_count().was_initialized = 2;
    expected_count().was_destructed = 2;
}

TEST_F(StaticVectorFixture, try_emplace_back_forwards_its_arguments) {
    constexpr size_t const VECTOR_CAPACITY = 5;
    iox2::bb::StaticVector<Observable, VECTOR_CAPACITY> sut;
    int32_t const tracking_id1 = 99;
    ASSERT_TRUE(sut.try_emplace_back(Observable { tracking_id1 }));
    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(sut.unchecked_access()[0].id, tracking_id1);
    int32_t const tracking_id2 = 222;
    Observable const obs { tracking_id2 };
    EXPECT_EQ(Observable::s_counter.was_initialized, 2);
    ASSERT_TRUE(sut.try_emplace_back(obs));
    EXPECT_EQ(Observable::s_counter.was_initialized, 2);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0].id, tracking_id1);
    EXPECT_EQ(sut.unchecked_access()[1].id, tracking_id2);
    ASSERT_EQ(obs.id, tracking_id2);
    expected_count().was_initialized = 2;
    expected_count().was_move_constructed = 1;
    expected_count().was_copy_constructed = 1;
    expected_count().was_destructed = 4;
}

TEST_F(StaticVectorFixture, try_emplace_back_fails_if_vector_is_full) {
    constexpr size_t const SMALL_VECTOR_CAPACITY = 1;
    iox2::bb::StaticVector<Observable, SMALL_VECTOR_CAPACITY> sut;
    int32_t const tracking_id = 99;
    ASSERT_TRUE(sut.try_emplace_back(tracking_id));
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(sut.unchecked_access()[0].id, tracking_id);
    ASSERT_FALSE(sut.try_emplace_back());
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(sut.unchecked_access()[0].id, tracking_id);
    expected_count().was_initialized = 1;
    expected_count().was_move_constructed = 0;
    expected_count().was_copy_constructed = 0;
    expected_count().was_destructed = 1;
}

TEST(StaticVector, try_emplace_at_inserts_elements_in_the_middle_of_vector) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(3));
    ASSERT_TRUE(sut.try_emplace_at(1, 2));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_emplace_at_inserts_elements_at_the_front_of_vector) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_emplace_back(3));
    ASSERT_TRUE(sut.try_emplace_at(0, 1));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_emplace_at_inserts_elements_at_the_back_of_vector) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_emplace_at(2, 3));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_emplace_at_fails_if_vector_is_full_leaving_contents_intact) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_emplace_back(3));
    ASSERT_FALSE(sut.try_emplace_at(1, 0));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_emplace_at_fails_if_index_is_invalid_leaving_contents_intact) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_emplace_at(3, 3));
    ASSERT_FALSE(sut.try_emplace_at(4, 3));
    ASSERT_FALSE(sut.try_emplace_at(5, 3));
    ASSERT_EQ(sut.size(), 2);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
}

TEST(StaticVector, try_erase_at_removes_element_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.try_erase_at(4));
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE - 1);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    ASSERT_TRUE(sut.try_erase_at(1));
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[3]);
    ASSERT_TRUE(sut.try_erase_at(0));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[3]);
    ASSERT_TRUE(sut.try_erase_at(0));
    ASSERT_TRUE(sut.try_erase_at(0));
    ASSERT_EQ(sut.size(), 0);
    ASSERT_TRUE(sut.empty());
}

TEST(StaticVector, try_erase_at_fails_for_invalid_index) {
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(G_TEST_ARRAY);
        ASSERT_FALSE(sut.try_erase_at(G_TEST_ARRAY_SIZE));
        ASSERT_FALSE(sut.try_erase_at(G_TEST_ARRAY_SIZE + 1));
        ASSERT_FALSE(sut.try_erase_at(G_TEST_ARRAY_SIZE + 2));
        ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
        EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
        EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
        EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
        EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
        EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
    }
    {
        iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut;
        ASSERT_FALSE(sut.try_erase_at(0));
        ASSERT_FALSE(sut.try_erase_at(1));
        ASSERT_FALSE(sut.try_erase_at(2));
        ASSERT_TRUE(sut.empty());
    }
}

TEST(StaticVector, try_erase_at_removes_range_of_elements_from_middle) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.try_erase_at(1, 3));
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[4]);
}

TEST(StaticVector, try_erase_at_removes_range_of_elements_from_back) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.try_erase_at(3, G_TEST_ARRAY_SIZE));
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
}

TEST(StaticVector, try_erase_at_removes_range_of_elements_from_front) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.try_erase_at(0, 3));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[4]);
}

TEST(StaticVector, try_erase_at_removes_range_of_elements_entire_range) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.try_erase_at(0, G_TEST_ARRAY_SIZE));
    ASSERT_EQ(sut.size(), 0);
}

TEST(StaticVector, try_erase_at_removes_range_of_elements_empty_range) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.try_erase_at(0, 0));
    ASSERT_TRUE(sut.try_erase_at(1, 1));
    ASSERT_TRUE(sut.try_erase_at(2, 2));
    ASSERT_TRUE(sut.try_erase_at(3, 3));
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
}

TEST(StaticVector, try_erase_at_fails_for_invalid_start_index_leaving_contents_intact) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_FALSE(sut.try_erase_at(1, 0));
    ASSERT_FALSE(sut.try_erase_at(2, 0));
    ASSERT_FALSE(sut.try_erase_at(2, 1));
    ASSERT_FALSE(sut.try_erase_at(3, 0));
    ASSERT_FALSE(sut.try_erase_at(3, 1));
    ASSERT_FALSE(sut.try_erase_at(3, 2));
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
}

TEST(StaticVector, try_erase_at_fails_for_invalid_end_index_leaving_contents_intact) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_FALSE(sut.try_erase_at(0, 6));
    ASSERT_FALSE(sut.try_erase_at(0, 7));
    ASSERT_FALSE(sut.try_erase_at(0, 8));
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
}

TEST(StaticVector, try_insert_at_inserts_elements_in_the_middle_of_vector) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(3));
    ASSERT_TRUE(sut.try_insert_at(1, 2));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_insert_at_inserts_elements_at_the_front_of_vector) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_emplace_back(3));
    ASSERT_TRUE(sut.try_insert_at(0, 1));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_insert_at_inserts_elements_at_the_back_of_vector) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at(2, 3));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_insert_at_fails_if_vector_is_full_leaving_contents_intact) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_emplace_back(3));
    ASSERT_FALSE(sut.try_insert_at(1, 0));
    ASSERT_EQ(sut.size(), 3);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
    ASSERT_EQ(sut.unchecked_access()[2], 3);
}

TEST(StaticVector, try_insert_at_fails_if_index_is_invalid_leaving_contents_intact) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_insert_at(3, 3));
    ASSERT_FALSE(sut.try_insert_at(4, 3));
    ASSERT_FALSE(sut.try_insert_at(5, 3));
    ASSERT_EQ(sut.size(), 2);
    ASSERT_EQ(sut.unchecked_access()[0], 1);
    ASSERT_EQ(sut.unchecked_access()[1], 2);
}

TEST_F(StaticVectorFixture, try_insert_at_moves_elements_if_argument_is_rvalue) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<Observable, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(Observable {}));
    ASSERT_TRUE(sut.try_emplace_back(Observable {}));
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    ASSERT_TRUE(sut.try_insert_at(1, Observable {}));
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    expected_count().was_initialized = Observable::s_counter.was_initialized;
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_move_assigned = Observable::s_counter.was_move_assigned;
    expected_count().was_destructed =
        Observable::s_counter.was_initialized + Observable::s_counter.was_move_constructed;
}

TEST_F(StaticVectorFixture, try_insert_at_copies_elements_if_argument_is_lvalue) {
    constexpr size_t const VECTOR_CAPACITY = 3;
    iox2::bb::StaticVector<Observable, VECTOR_CAPACITY> sut;
    ASSERT_TRUE(sut.try_emplace_back(Observable {}));
    ASSERT_TRUE(sut.try_emplace_back(Observable {}));
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    int32_t const tracking_id = 12345;
    Observable const obj { tracking_id };
    ASSERT_TRUE(sut.try_insert_at(1, obj));
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 1);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    ASSERT_EQ(sut.unchecked_access()[1].id, tracking_id);
    ASSERT_EQ(obj.id, tracking_id);
    expected_count().was_initialized = Observable::s_counter.was_initialized;
    expected_count().was_move_constructed = Observable::s_counter.was_move_constructed;
    expected_count().was_move_assigned = Observable::s_counter.was_move_assigned;
    expected_count().was_copy_constructed = 1;
    expected_count().was_destructed =
        Observable::s_counter.was_initialized + Observable::s_counter.was_move_constructed + 1;
}

TEST(StaticVector, try_insert_at_inserting_range_of_elements_in_the_middle) {
    constexpr size_t const VECTOR_CAPACITY = 6;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    int32_t const inserted_value = 100;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at(1, 4, inserted_value));
    ASSERT_EQ(sut.size(), 6);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[2], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[3], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[4], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[5], 2);
}

TEST(StaticVector, try_insert_at_inserting_range_of_elements_at_the_front) {
    constexpr size_t const VECTOR_CAPACITY = 6;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    int32_t const inserted_value = 100;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at(0, 4, inserted_value));
    ASSERT_EQ(sut.size(), 6);
    EXPECT_EQ(sut.unchecked_access()[0], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[1], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[2], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[3], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[4], 1);
    EXPECT_EQ(sut.unchecked_access()[5], 2);
}

TEST(StaticVector, try_insert_at_inserting_range_of_elements_at_the_back) {
    constexpr size_t const VECTOR_CAPACITY = 6;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    int32_t const inserted_value = 100;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at(2, 4, inserted_value));
    ASSERT_EQ(sut.size(), 6);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
    EXPECT_EQ(sut.unchecked_access()[2], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[3], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[4], inserted_value);
    EXPECT_EQ(sut.unchecked_access()[5], inserted_value);
}

TEST(StaticVector, try_insert_at_inserting_range_fails_for_invalid_index_preserving_vector_contents) {
    constexpr size_t const VECTOR_CAPACITY = 10;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    int32_t const inserted_value = 100;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_insert_at(3, 4, inserted_value));
    ASSERT_FALSE(sut.try_insert_at(4, 4, inserted_value));
    ASSERT_FALSE(sut.try_insert_at(5, 4, inserted_value));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
}

TEST(StaticVector, try_insert_at_inserting_range_fails_if_count_exceeds_capacity_preserving_vector_contents) {
    constexpr size_t const VECTOR_CAPACITY = 10;
    iox2::bb::StaticVector<int32_t, VECTOR_CAPACITY> sut;
    int32_t const inserted_value = 100;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_insert_at(0, 9, inserted_value));
    ASSERT_FALSE(sut.try_insert_at(0, 10, inserted_value));
    ASSERT_FALSE(sut.try_insert_at(0, 11, inserted_value));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
}

TEST(StaticVector, try_insert_at_unchecked_inserts_a_range_of_elements_in_the_middle) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at_unchecked(1, std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY)));
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE + 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[3]);
    EXPECT_EQ(sut.unchecked_access()[5], G_TEST_ARRAY[4]);
    EXPECT_EQ(sut.unchecked_access()[6], 2);
}

TEST(StaticVector, try_insert_at_unchecked_inserts_a_range_of_elements_at_the_back) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at_unchecked(2, std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY)));
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE + 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut.unchecked_access()[5], G_TEST_ARRAY[3]);
    EXPECT_EQ(sut.unchecked_access()[6], G_TEST_ARRAY[4]);
}

TEST(StaticVector, try_insert_at_unchecked_inserts_a_range_of_elements_at_the_front) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at_unchecked(0, std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY)));
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE + 2);
    EXPECT_EQ(sut.unchecked_access()[0], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[1], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[3]);
    EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[4]);
    EXPECT_EQ(sut.unchecked_access()[5], 1);
    EXPECT_EQ(sut.unchecked_access()[6], 2);
}

TEST(StaticVector, try_insert_at_unchecked_fails_for_invalid_index_leaving_vector_contents_intact) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_insert_at_unchecked(3, std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY)));
    ASSERT_FALSE(sut.try_insert_at_unchecked(4, std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY)));
    ASSERT_FALSE(sut.try_insert_at_unchecked(5, std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY)));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
}

TEST(StaticVector, try_insert_at_unchecked_fails_for_exceeding_capacity_leaving_vector_contents_intact) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_insert_at_unchecked(0, std::begin(G_TEST_ARRAY), std::end(G_TEST_ARRAY)));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
}

TEST(StaticVector, try_insert_at_unchecked_inserts_an_init_list_in_the_middle) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at_unchecked(1, { G_TEST_ARRAY[0], G_TEST_ARRAY[1], G_TEST_ARRAY[2] }));
    ASSERT_EQ(sut.size(), 5);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut.unchecked_access()[4], 2);
}

TEST(StaticVector, try_insert_at_unchecked_inserts_init_list_at_the_back) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at_unchecked(2, { G_TEST_ARRAY[0], G_TEST_ARRAY[1], G_TEST_ARRAY[2] }));
    ASSERT_EQ(sut.size(), 5);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[4], G_TEST_ARRAY[2]);
}

TEST(StaticVector, try_insert_at_unchecked_inserts_init_list_at_the_front) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_TRUE(sut.try_insert_at_unchecked(0, { G_TEST_ARRAY[0], G_TEST_ARRAY[1], G_TEST_ARRAY[2] }));
    ASSERT_EQ(sut.size(), 5);
    EXPECT_EQ(sut.unchecked_access()[0], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[1], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut.unchecked_access()[3], 1);
    EXPECT_EQ(sut.unchecked_access()[4], 2);
}

TEST(StaticVector, try_insert_at_unchecked_init_list_fails_for_invalid_index_leaving_vector_contents_intact) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_insert_at_unchecked(3, { G_TEST_ARRAY[0], G_TEST_ARRAY[1], G_TEST_ARRAY[2] }));
    ASSERT_FALSE(sut.try_insert_at_unchecked(4, { G_TEST_ARRAY[0], G_TEST_ARRAY[1], G_TEST_ARRAY[2] }));
    ASSERT_FALSE(sut.try_insert_at_unchecked(5, { G_TEST_ARRAY[0], G_TEST_ARRAY[1], G_TEST_ARRAY[2] }));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
}

TEST(StaticVector, try_insert_at_unchecked_init_list_fails_for_exceeding_capacity_leaving_vector_contents_intact) {
    iox2::bb::StaticVector<int32_t, 4> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_TRUE(sut.try_emplace_back(2));
    ASSERT_FALSE(sut.try_insert_at_unchecked(0, { G_TEST_ARRAY[0], G_TEST_ARRAY[1], G_TEST_ARRAY[2] }));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], 1);
    EXPECT_EQ(sut.unchecked_access()[1], 2);
}

TEST(StaticVector, clear_removes_all_elements) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(G_TEST_ARRAY);
    sut.clear();
    ASSERT_TRUE(sut.empty());
    ASSERT_EQ(sut.size(), 0);
}

TEST(StaticVector, try_pop_back_removes_last_element) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 4);
    EXPECT_EQ(sut.unchecked_access()[0], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[1], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[2]);
    EXPECT_EQ(sut.unchecked_access()[3], G_TEST_ARRAY[3]);
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(sut.unchecked_access()[0], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[1], G_TEST_ARRAY[1]);
    EXPECT_EQ(sut.unchecked_access()[2], G_TEST_ARRAY[2]);
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(sut.unchecked_access()[0], G_TEST_ARRAY[0]);
    EXPECT_EQ(sut.unchecked_access()[1], G_TEST_ARRAY[1]);
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(sut.unchecked_access()[0], G_TEST_ARRAY[0]);
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_TRUE(sut.empty());
    ASSERT_EQ(sut.size(), 0);
    ASSERT_FALSE(sut.try_pop_back());
    ASSERT_TRUE(sut.empty());
    ASSERT_EQ(sut.size(), 0);
    ASSERT_FALSE(sut.try_pop_back());
}

TEST(StaticVector, capacity_retuns_capacity) {
    ASSERT_EQ((iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> { G_TEST_ARRAY }.capacity()), G_TEST_ARRAY_SIZE);
    ASSERT_EQ((iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> {}.capacity()), G_TEST_ARRAY_SIZE + 1);
    ASSERT_EQ((iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 2> {}.capacity()), G_TEST_ARRAY_SIZE + 2);
}

TEST(StaticVector, element_at_retrieves_mutable_reference_to_element_at_index) {
    iox2::bb::StaticVector<int32_t, 4> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_EQ(sut.size(), 1);
    ASSERT_TRUE(sut.element_at(0));
    EXPECT_EQ(*sut.element_at(0), 1);
    sut.element_at(0)->get() = 2;
    ASSERT_TRUE(sut.element_at(0));
    EXPECT_EQ(*sut.element_at(0), 2);
}

TEST(StaticVector, element_at_fails_for_invalid_index) {
    iox2::bb::StaticVector<int32_t, 4> sut;
    ASSERT_TRUE(sut.try_emplace_back(1));
    ASSERT_FALSE(sut.element_at(1));
    ASSERT_FALSE(sut.element_at(2));
    sut.clear();
    ASSERT_FALSE(sut.element_at(0));
}

TEST(StaticVector, element_at_retrieves_immutable_reference_from_const_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    ASSERT_TRUE(sut.element_at(0));
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    ASSERT_TRUE(sut.element_at(1));
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    ASSERT_TRUE(sut.element_at(2));
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    ASSERT_TRUE(sut.element_at(3));
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    ASSERT_TRUE(sut.element_at(4));
    EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
}

TEST(StaticVector, element_at_fails_for_invalid_index_from_const_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    ASSERT_FALSE(sut.element_at(5));
    ASSERT_FALSE(sut.element_at(6));
    ASSERT_FALSE(sut.element_at(7));
}

TEST(StaticVector, front_element_returns_mutable_reference_to_first_element) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.front_element());
    EXPECT_EQ(*sut.front_element(), G_TEST_ARRAY[0]);
    sut.front_element()->get() += 2;
    ASSERT_TRUE(sut.front_element());
    EXPECT_EQ(sut.unchecked_access()[0], G_TEST_ARRAY[0] + 2);
}

TEST(StaticVector, front_element_fails_for_empty_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut;
    iox2::bb::testing::opaque_use(sut);
    ASSERT_FALSE(sut.front_element());
}

TEST(StaticVector, front_element_const_returns_first_element) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.front_element());
    EXPECT_EQ(*sut.front_element(), G_TEST_ARRAY[0]);
}

TEST(StaticVector, front_element_const_fails_for_empty_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut;
    ASSERT_FALSE(sut.front_element());
}

TEST(StaticVector, back_element_returns_mutable_reference_to_first_element) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.back_element());
    EXPECT_EQ(*sut.back_element(), G_TEST_ARRAY[G_TEST_ARRAY_SIZE - 1]);
    sut.back_element()->get() += 2;
    ASSERT_TRUE(sut.back_element());
    EXPECT_EQ(sut.unchecked_access()[G_TEST_ARRAY_SIZE - 1], G_TEST_ARRAY[G_TEST_ARRAY_SIZE - 1] + 2);
}

TEST(StaticVector, back_element_fails_for_empty_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut;
    iox2::bb::testing::opaque_use(sut);
    ASSERT_FALSE(sut.back_element());
}

TEST(StaticVector, back_element_const_returns_first_element) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut(G_TEST_ARRAY);
    ASSERT_TRUE(sut.back_element());
    EXPECT_EQ(*sut.back_element(), G_TEST_ARRAY[G_TEST_ARRAY_SIZE - 1]);
}

TEST(StaticVector, back_element_const_fails_for_empty_vector) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut;
    ASSERT_FALSE(sut.back_element());
}

TEST(StaticVector, unchecked_const_array_access) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut(G_TEST_ARRAY);
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(&sut.unchecked_access()[0], &sut.element_at(0).value().get());
}

TEST(StaticVector, unchecked_const_begin_iterator) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.unchecked_access().begin(), &sut.element_at(0).value().get());
}

TEST(StaticVector, unchecked_const_end_iterator) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.unchecked_access().end(), std::next(sut.unchecked_access().begin(), G_TEST_ARRAY_SIZE));
}

TEST(StaticVector, unchecked_const_data_pointer) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.unchecked_access().data(), &sut.element_at(0).value().get());
}

TEST(StaticVector, unchecked_mutable_array_access) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(&sut.unchecked_access()[0], &sut.element_at(0).value().get());
    sut.unchecked_access()[0] *= 2;
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0] * 2);
}

TEST(StaticVector, unchecked_mutable_begin_iterator) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.unchecked_access().begin(), &sut.element_at(0).value().get());
    *sut.unchecked_access().begin() *= 2;
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0] * 2);
}

TEST(StaticVector, unchecked_mutable_end_iterator) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.unchecked_access().end(), std::next(sut.unchecked_access().begin(), G_TEST_ARRAY_SIZE));
    *(std::prev(sut.unchecked_access().end())) *= 2;
    EXPECT_EQ(*sut.element_at(G_TEST_ARRAY_SIZE - 1), G_TEST_ARRAY[G_TEST_ARRAY_SIZE - 1] * 2);
}

TEST(StaticVector, unchecked_mutable_data_pointer) {
    iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    ASSERT_EQ(sut.unchecked_access().data(), &sut.element_at(0).value().get());
    *sut.unchecked_access().data() *= 2;
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0] * 2);
}

TEST(StaticVector, equality_comparison) {
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                 == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 0, 2, 3 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 0, 2, 3 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 0, 3 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 0, 3 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 0 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 0 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3, 4 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3, 4 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1 })
                  == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 2 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1 })
                 == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({})
                 == *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({})));
}

TEST(StaticVector, not_equal_comparison) {
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                  != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 0, 2, 3 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 0, 2, 3 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 0, 3 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 0, 3 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 0 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 0 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3, 4 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1, 2, 3, 4 })));
    ASSERT_TRUE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1 })
                 != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 2 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1 })
                  != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({ 1 })));
    ASSERT_FALSE((*iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({})
                  != *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_initializer_list({})));
}

TEST(StaticVector, ostream_insertion_converts_contents_to_string) {
    std::ostringstream sstr;
    auto sut = *iox2::bb::StaticVector<int32_t, G_TEST_ARRAY_SIZE>::from_range_unchecked(G_TEST_ARRAY);
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 5, m_data: [ 4, 9, 77, 32, -5 ] }");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_TRUE(sut.try_pop_back());
    sstr = std::ostringstream {};
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 3, m_data: [ 4, 9, 77 ] }");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_TRUE(sut.try_pop_back());
    sstr = std::ostringstream {};
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 1, m_data: [ 4 ] }");
    sut.clear();
    sstr = std::ostringstream {};
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 0, m_data: [  ] }");
}
} // namespace

// NOTE: the class needs to be outside to the anonymous namespace, else MSVC is not able to find the ostream operator
class StaticVectorePrintable {
    static int32_t s_print_count;

  public:
    static void reset_print_count() {
        s_print_count = 0;
    }
    friend auto operator<<(std::ostream& ostr, StaticVectorePrintable const& /*unused*/) -> std::ostream& {
        return ostr << ++s_print_count;
    }
};
int32_t StaticVectorePrintable::s_print_count = 0;

namespace {
TEST(StaticVector, ostream_insertion_calls_ostream_inserter_for_values) {
    constexpr size_t const VECTOR_CAPACITY = 5;
    iox2::bb::StaticVector<StaticVectorePrintable, VECTOR_CAPACITY> sut;
    StaticVectorePrintable::reset_print_count();
    std::ostringstream sstr;
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 0, m_data: [  ] }");
    ASSERT_TRUE(sut.try_emplace_back());
    sstr = std::ostringstream {};
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 1, m_data: [ 1 ] }");
    ASSERT_TRUE(sut.try_emplace_back());
    ASSERT_TRUE(sut.try_emplace_back());
    ASSERT_TRUE(sut.try_emplace_back());
    ASSERT_TRUE(sut.try_emplace_back());
    sstr = std::ostringstream {};
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 5, m_data: [ 2, 3, 4, 5, 6 ] }");
    sstr = std::ostringstream {};
    sstr << sut;
    ASSERT_TRUE(sstr);
    EXPECT_EQ(sstr.str(), "StaticVector::<5> { m_size: 5, m_data: [ 7, 8, 9, 10, 11 ] }");
}

} // namespace
