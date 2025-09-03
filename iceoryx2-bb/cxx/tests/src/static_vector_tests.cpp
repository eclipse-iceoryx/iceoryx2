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

#include "iox2/container/static_vector.hpp"

#include "testing/observable.hpp"
#include "testing/test_utils.hpp"

#include "gtest/gtest.h"

namespace {
using iox2::container::testing::Observable;

class StaticVectorFixture : public iox2::container::testing::VerifyAllObservableInteractionsFixture { };

// NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
constexpr size_t const G_TEST_ARRAY_SIZE = 5;
// NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays)
int32_t const G_TEST_ARRAY[G_TEST_ARRAY_SIZE] = { 4, 9, 77, 32, -5 };


// NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
static_assert(std::is_standard_layout<iox2::container::StaticVector<int32_t, G_TEST_ARRAY_SIZE>>::value,
              "StaticVector must be standard layout");

TEST(StaticVector, default_constructor_initializes_to_empty) {
    iox2::container::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut;
    ASSERT_TRUE(sut.empty());
}

TEST_F(StaticVectorFixture, default_constructor_does_not_construct_any_objects) {
    iox2::container::StaticVector<Observable, G_TEST_ARRAY_SIZE> const sut;
    ASSERT_TRUE(sut.empty());
}

TEST(StaticVector, array_constructor_copies_array_elements_into_vector) {
    iox2::container::StaticVector<int32_t, G_TEST_ARRAY_SIZE> const sut(G_TEST_ARRAY);
    ASSERT_TRUE(!sut.empty());
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_EQ(*sut.element_at(0), G_TEST_ARRAY[0]);
    EXPECT_EQ(*sut.element_at(1), G_TEST_ARRAY[1]);
    EXPECT_EQ(*sut.element_at(2), G_TEST_ARRAY[2]);
    EXPECT_EQ(*sut.element_at(3), G_TEST_ARRAY[3]);
    EXPECT_EQ(*sut.element_at(4), G_TEST_ARRAY[4]);
}

TEST(StaticVector, array_constructor_leaves_uninitialized_elements_up_to_capacity) {
    iox2::container::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> const sut(G_TEST_ARRAY);
    ASSERT_TRUE(!sut.empty());
    ASSERT_EQ(sut.size(), G_TEST_ARRAY_SIZE);
    EXPECT_TRUE(!sut.element_at(G_TEST_ARRAY_SIZE).has_value());
}

TEST(StaticVector, try_push_back_inserts_elements_at_the_back_if_there_is_room) {
    iox2::container::StaticVector<int32_t, G_TEST_ARRAY_SIZE + 1> sut(G_TEST_ARRAY);
    int32_t const test_value = 99;
    ASSERT_TRUE(sut.try_push_back(test_value));
    EXPECT_EQ(*sut.element_at(G_TEST_ARRAY_SIZE), test_value);
}

TEST(StaticVector, try_push_back_returns_false_if_there_is_no_room) {
    iox2::container::StaticVector<int32_t, G_TEST_ARRAY_SIZE> sut(G_TEST_ARRAY);
    int32_t const test_value = 99;
    ASSERT_TRUE(!sut.try_push_back(test_value));
}

} // namespace
