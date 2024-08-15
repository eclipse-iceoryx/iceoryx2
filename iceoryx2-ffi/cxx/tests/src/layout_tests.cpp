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

#include "iox/layout.hpp"

#include "test.hpp"

namespace {
using namespace iox;

template <typename T>
auto from_type_works() {
    auto sut = Layout::from<T>();

    ASSERT_EQ(sut.size(), sizeof(T));
    ASSERT_EQ(sut.alignment(), alignof(T));
}

TEST(Layout, from_type_works) {
    from_type_works<uint8_t>();
    from_type_works<uint16_t>();
    from_type_works<uint32_t>();
    from_type_works<uint64_t>();
}

TEST(Layout, from_void_works) {
    auto sut = Layout::from<void>();
    ASSERT_EQ(sut.size(), 0);
    ASSERT_EQ(sut.alignment(), 1);
}

TEST(Layout, create_with_correct_size_align_works) {
    constexpr uint64_t SIZE = 32;
    constexpr uint64_t ALIGN = 8;

    auto sut = Layout::create(SIZE, ALIGN);

    ASSERT_TRUE(sut.has_value());
    ASSERT_EQ(sut->size(), SIZE);
    ASSERT_EQ(sut->alignment(), ALIGN);
}

TEST(Layout, create_with_misaligned_size_and_correct_align_works) {
    constexpr uint64_t ALIGNED_SIZE = 32;
    constexpr uint64_t MISALIGNED_SIZE = 19;
    constexpr uint64_t ALIGN = 16;

    auto sut = Layout::create(MISALIGNED_SIZE, ALIGN);

    ASSERT_TRUE(sut.has_value());
    ASSERT_EQ(sut->size(), ALIGNED_SIZE);
    ASSERT_EQ(sut->alignment(), ALIGN);
}

TEST(Layout, create_with_size_zero_and_correct_align_works) {
    constexpr uint64_t SIZE = 0;
    constexpr uint64_t ALIGN = 16;

    auto sut = Layout::create(SIZE, ALIGN);

    ASSERT_TRUE(sut.has_value());
    ASSERT_EQ(sut->size(), SIZE);
    ASSERT_EQ(sut->alignment(), ALIGN);
}

TEST(Layout, create_with_invalid_alignment_fails) {
    constexpr uint64_t SIZE = 8;
    constexpr uint64_t ALIGN_NOT_POWER_OF_TWO = 5;

    auto sut = Layout::create(SIZE, ALIGN_NOT_POWER_OF_TWO);

    ASSERT_TRUE(sut.has_error());
    ASSERT_EQ(sut.error(), LayoutCreationError::InvalidAlignment);
}
} // namespace
