// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#include "iox/slice.hpp"
#include "test.hpp"

#include <array>

namespace {

struct DummyData {
    static constexpr uint64_t DEFAULT_VALUE_A = 42;
    static constexpr bool DEFAULT_VALUE_Z { false };
    uint64_t a { DEFAULT_VALUE_A };
    bool z { DEFAULT_VALUE_Z };
};

TEST(SliceTest, const_correctness_is_maintained) {
    constexpr uint64_t SLICE_MAX_LENGTH = 10;

    auto elements = std::array<DummyData, SLICE_MAX_LENGTH> {};

    auto mutable_slice = iox::MutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    ASSERT_FALSE(std::is_const_v<std::remove_pointer_t<decltype(mutable_slice.begin())>>);
    ASSERT_FALSE(std::is_const_v<std::remove_pointer_t<decltype(mutable_slice.end())>>);
    ASSERT_FALSE(std::is_const_v<std::remove_pointer_t<decltype(mutable_slice.data())>>);
    ASSERT_FALSE(std::is_const_v<std::remove_reference_t<decltype(mutable_slice[0])>>);

    // const instances of MutableSlice are also not mutable
    const auto const_mutable_slice = iox::MutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(const_mutable_slice.begin())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(const_mutable_slice.end())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(const_mutable_slice.data())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_reference_t<decltype(const_mutable_slice[0])>>);

    auto immutable_slice = iox::ImmutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(immutable_slice.begin())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(immutable_slice.end())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(immutable_slice.data())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_reference_t<decltype(immutable_slice[0])>>);

    const auto const_immutable_slice = iox::ImmutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(const_immutable_slice.begin())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(const_immutable_slice.end())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_pointer_t<decltype(const_immutable_slice.data())>>);
    ASSERT_TRUE(std::is_const_v<std::remove_reference_t<decltype(const_immutable_slice[0])>>);
}

TEST(SliceTest, can_iterate_mutable_slice) {
    constexpr uint64_t SLICE_MAX_LENGTH = 10;

    auto elements = std::array<DummyData, SLICE_MAX_LENGTH> {};

    auto mutable_slice = iox::MutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    auto iterations = 0;
    for (auto element : mutable_slice) {
        ASSERT_THAT(element.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(element.z, Eq(DummyData::DEFAULT_VALUE_Z));
        iterations++;
    }
    ASSERT_EQ(iterations, SLICE_MAX_LENGTH);
}

TEST(SliceTest, can_iterate_const_mutable_slice) {
    constexpr uint64_t SLICE_MAX_LENGTH = 10;

    auto elements = std::array<DummyData, SLICE_MAX_LENGTH> {};

    const auto slice = iox::MutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    auto iterations = 0;
    for (auto element : slice) {
        ASSERT_THAT(element.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(element.z, Eq(DummyData::DEFAULT_VALUE_Z));
        iterations++;
    }
    ASSERT_EQ(iterations, SLICE_MAX_LENGTH);
}

TEST(SliceTest, can_iterate_immutable_slice) {
    constexpr uint64_t SLICE_MAX_LENGTH = 10;

    auto elements = std::array<DummyData, SLICE_MAX_LENGTH> {};

    auto slice = iox::ImmutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    auto iterations = 0;
    for (auto element : slice) {
        ASSERT_THAT(element.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(element.z, Eq(DummyData::DEFAULT_VALUE_Z));
        iterations++;
    }
    ASSERT_EQ(iterations, SLICE_MAX_LENGTH);
}

TEST(SliceTest, can_iterate_const_immutable_slice) {
    constexpr uint64_t SLICE_MAX_LENGTH = 10;

    auto elements = std::array<DummyData, SLICE_MAX_LENGTH> {};

    const auto slice = iox::ImmutableSlice<DummyData>(elements.data(), SLICE_MAX_LENGTH);
    auto iterations = 0;
    for (auto element : slice) {
        ASSERT_THAT(element.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(element.z, Eq(DummyData::DEFAULT_VALUE_Z));
        iterations++;
    }
    ASSERT_EQ(iterations, SLICE_MAX_LENGTH);
}

} // namespace
