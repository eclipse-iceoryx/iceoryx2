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

#include "iox2/container/static_string.hpp"

#include "testing/test_utils.hpp"

#include "gtest/gtest.h"

#include <limits>

namespace {

// Use the detection idiom to check if the static array bounds check is evaluated correctly
// see https://en.cppreference.com/w/cpp/types/void_t.html
template <class...>
using DetectT = void;

constexpr uint64_t const G_ARBITRARY_CAPACITY = 55;
// NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
static_assert(std::is_standard_layout<iox2::container::StaticString<G_ARBITRARY_CAPACITY>>::value,
              "StaticString must be standard layout");
static_assert(iox2::container::StaticString<G_ARBITRARY_CAPACITY>::capacity() == G_ARBITRARY_CAPACITY,
              "Capacity must be determined by template argument");

TEST(StaticString, default_constructor_initializes_to_empty) {
    constexpr uint64_t const STRING_SIZE = 5;
    iox2::container::StaticString<STRING_SIZE> const sut;
    ASSERT_TRUE(sut.empty());
    ASSERT_EQ(sut.size(), 0);
}

TEST(StaticString, from_utf8_construction_from_c_style_ascii_string) {
    constexpr uint64_t const STRING_SIZE = 15;
    auto const opt_sut = iox2::container::StaticString<STRING_SIZE>::from_utf8("hello world!");
    ASSERT_TRUE(opt_sut.has_value());
    auto const& sut = opt_sut.value();
    ASSERT_EQ(sut.size(), 12);
}

TEST(StaticString, from_utf8_fails_if_string_is_not_null_terminated) {
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) testing
    char const array_not_null_terminated[] = { 'A', 'B', 'C' };
    constexpr uint64_t const STRING_SIZE = 15;
    auto const opt_sut = iox2::container::StaticString<STRING_SIZE>::from_utf8(array_not_null_terminated);
    ASSERT_TRUE(!opt_sut.has_value());
}

template <bool IsSigned = std::numeric_limits<char>::is_signed>
struct InvalidChar;
template <>
struct InvalidChar<true> : std::integral_constant<int, std::numeric_limits<char>::min()> { };
template <>
struct InvalidChar<false> : std::integral_constant<int, std::numeric_limits<char>::max()> { };

TEST(StaticString, from_utf8_fails_if_string_has_invalid_characters) {
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) testing
    char input_array[] = { 'A', 'B', 'C', '\0' };
    char const invalid_character = static_cast<char>(InvalidChar<>::value);
    constexpr uint64_t const STRING_SIZE = 15;
    ASSERT_TRUE(iox2::container::StaticString<STRING_SIZE>::from_utf8(input_array).has_value());
    input_array[0] = invalid_character;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8(input_array).has_value());
    input_array[0] = 'A';
    input_array[1] = invalid_character;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8(input_array).has_value());
    input_array[1] = 'B';
    input_array[2] = invalid_character;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8(input_array).has_value());
}


template <uint64_t, class = void>
struct DetectInvalidFromUtf8WithStringABC : std::false_type { };
template <uint64_t M>
struct DetectInvalidFromUtf8WithStringABC<M, DetectT<decltype(iox2::container::StaticString<M>::from_utf8("ABC"))>>
    : std::true_type { };

TEST(StaticString, from_utf8_works_up_to_capacity) {
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) testing
    char const test_string[] = { 'A', 'B', 'C', '\0' };
    constexpr uint64_t const STRING_SIZE = 3;
    auto const opt_sut = iox2::container::StaticString<STRING_SIZE>::from_utf8(test_string);
    ASSERT_TRUE(opt_sut.has_value());
    ASSERT_STREQ(opt_sut->unchecked_access().c_str(), "ABC");
    static_assert(DetectInvalidFromUtf8WithStringABC<4>::value, "ABC fits into capacity 4");
    static_assert(DetectInvalidFromUtf8WithStringABC<3>::value, "ABC fits into capacity 3");
    static_assert(!DetectInvalidFromUtf8WithStringABC<2>::value, "ABC does not fit into capacity 2");
    static_assert(!DetectInvalidFromUtf8WithStringABC<1>::value, "ABC does not fit into capacity 1");
    static_assert(!DetectInvalidFromUtf8WithStringABC<0>::value, "ABC does not fit into capacity 0");
}

// NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) capacity has no significance for this test
template <typename T, typename U = decltype(iox2::container::StaticString<99>::from_utf8(std::declval<T&&>()))>
constexpr auto can_call_from_utf8_with(T&& /* unused */) -> std::true_type {
    return {};
}
// NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
template <typename T, typename = std::enable_if_t<!std::is_array<std::remove_reference_t<T>>::value, bool>>
constexpr auto can_call_from_utf8_with(T&& /* unused */) -> std::false_type {
    return {};
}

TEST(StaticString, from_utf8_works_only_with_statically_known_strings) {
    static_assert(can_call_from_utf8_with("ABC"));
    static_assert(!can_call_from_utf8_with(static_cast<char const*>("ABC")));
}

TEST(StaticString, from_utf8_null_terminated_unchecked_construction_from_null_terminated_c_style_string) {
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) testing
    char const test_string[] = "Hello World";
    constexpr uint64_t const STRING_SIZE = 15;
    auto const opt_sut = iox2::container::StaticString<STRING_SIZE>::from_utf8(test_string);
    ASSERT_TRUE(opt_sut.has_value());
    ASSERT_EQ(opt_sut->size(), sizeof(test_string) - 1);
    EXPECT_STREQ(opt_sut->unchecked_access().c_str(), static_cast<char const*>(test_string));
}

TEST(StaticString, from_utf8_null_terminated_unchecked_fails_if_string_has_invalid_characters) {
    // NOLINTBEGIN(clang-analyzer-security.insecureAPI.strcpy,cppcoreguidelines-pro-bounds-array-to-pointer-decay,hicpp-no-array-decay) testing
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) testing
    char const test_string[] = "Hello World";
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) testing
    char mutable_string[sizeof(test_string)];
    strcpy(mutable_string, test_string);
    char const* str_ptr = mutable_string;
    constexpr uint64_t const STRING_SIZE = 15;
    ASSERT_TRUE(iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked(str_ptr).has_value());
    mutable_string[0] = InvalidChar<>::value;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked(str_ptr).has_value());
    strcpy(mutable_string, test_string);
    mutable_string[1] = InvalidChar<>::value;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked(str_ptr).has_value());
    strcpy(mutable_string, test_string);
    mutable_string[2] = InvalidChar<>::value;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked(str_ptr).has_value());
    strcpy(mutable_string, test_string);
    mutable_string[3] = InvalidChar<>::value;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked(str_ptr).has_value());
    strcpy(mutable_string, test_string);
    mutable_string[sizeof(test_string) - 3] = InvalidChar<>::value;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked(str_ptr).has_value());
    strcpy(mutable_string, test_string);
    mutable_string[sizeof(test_string) - 2] = InvalidChar<>::value;
    ASSERT_TRUE(!iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked(str_ptr).has_value());
    // NOLINTEND(clang-analyzer-security.insecureAPI.strcpy,cppcoreguidelines-pro-bounds-array-to-pointer-decay,hicpp-no-array-decay)
}

TEST(StaticString, copy_constructor_copies_string_contents) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const test_string = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    iox2::container::StaticString<STRING_SIZE> const sut { test_string };
    ASSERT_EQ(sut.size(), 4);
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCD");
}

TEST(StaticString, move_constructor_copies_string_contents) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto test_string = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    iox2::container::StaticString<STRING_SIZE> const sut { std::move(test_string) };
    ASSERT_EQ(sut.size(), 4);
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCD");
}

TEST(StaticString, copy_assignment_copies_string_contents) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const test_string = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("EFGHI");
    EXPECT_STREQ(sut.unchecked_access().c_str(), "EFGHI");
    sut = test_string;
    ASSERT_EQ(sut.size(), 4);
    ASSERT_EQ(sut.unchecked_access()[4], '\0');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCD");
}

TEST(StaticString, copy_assignment_does_not_change_value_on_self_assignment) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    auto* sut_again_but_we_confuse_the_compiler = &sut;
    iox2::container::testing::opaque_use(static_cast<void*>(&sut_again_but_we_confuse_the_compiler));
    ASSERT_EQ(sut.size(), 4);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABCD");
    sut = *sut_again_but_we_confuse_the_compiler;
    ASSERT_EQ(sut.size(), 4);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABCD");
}

TEST(StaticString, copy_assignment_returns_reference_to_self) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const test_string = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("EFGHI");
    ASSERT_EQ(&(sut = test_string), &sut);
}

TEST(StaticString, move_assignment_copies_string_contents) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto test_string = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("EFGHI");
    EXPECT_STREQ(sut.unchecked_access().c_str(), "EFGHI");
    sut = std::move(test_string);
    ASSERT_EQ(sut.size(), 4);
    ASSERT_EQ(sut.unchecked_access()[4], '\0');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCD");
}

TEST(StaticString, move_assignment_returns_reference_to_self) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto test_string = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("EFGHI");
    ASSERT_EQ(&(sut = std::move(test_string)), &sut);
}

TEST(StaticString, construction_from_smaller_capacity_copies_string_contents) {
    constexpr uint64_t const SOURCE_STRING_SIZE = 4;
    auto const test_string = *iox2::container::StaticString<SOURCE_STRING_SIZE>::from_utf8("ABCD");
    constexpr uint64_t const DESTINATION_STRING_SIZE = 5;
    iox2::container::StaticString<DESTINATION_STRING_SIZE> const sut { test_string };
    ASSERT_EQ(sut.size(), 4);
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCD");
}


// NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) capacity has no significance for this test
template <uint64_t TargetCapacity,
          typename T,
          typename U = decltype(iox2::container::StaticString<TargetCapacity>(std::declval<T&&>()))>
constexpr auto can_construct_from(T&& /* unused */) -> std::true_type {
    return {};
}
// NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
template <uint64_t TargetCapacity,
          typename T,
          typename = std::enable_if_t<(std::remove_reference_t<T>::capacity() > TargetCapacity), bool>>
constexpr auto can_construct_from(T&& /* unused */) -> std::false_type {
    return {};
}


TEST(StaticString, construction_from_bigger_capacity_fails_regardless_of_content) {
    constexpr uint64_t const DESTINATION_STRING_SIZE = 5;
    ASSERT_TRUE(can_construct_from<DESTINATION_STRING_SIZE>(*iox2::container::StaticString<3>::from_utf8("A")));
    ASSERT_TRUE(can_construct_from<DESTINATION_STRING_SIZE>(*iox2::container::StaticString<4>::from_utf8("A")));
    ASSERT_TRUE(can_construct_from<DESTINATION_STRING_SIZE>(*iox2::container::StaticString<5>::from_utf8("A")));
    ASSERT_FALSE(can_construct_from<DESTINATION_STRING_SIZE>(*iox2::container::StaticString<6>::from_utf8("A")));
    ASSERT_FALSE(can_construct_from<DESTINATION_STRING_SIZE>(*iox2::container::StaticString<7>::from_utf8("A")));
}

TEST(StaticString, assignment_from_smaller_capacity_copies_string_contents) {
    constexpr uint64_t const SOURCE_STRING_SIZE = 4;
    auto const test_string = *iox2::container::StaticString<SOURCE_STRING_SIZE>::from_utf8("ABCD");
    constexpr uint64_t const DESTINATION_STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<DESTINATION_STRING_SIZE>::from_utf8("GHIJK");
    ASSERT_EQ(sut.size(), 5);
    sut = test_string;
    ASSERT_EQ(sut.size(), 4);
    ASSERT_EQ(sut.unchecked_access()[4], '\0');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCD");
}

TEST(StaticString, assignment_from_smaller_capacity_returns_reference_to_self) {
    constexpr uint64_t const SOURCE_STRING_SIZE = 4;
    auto const test_string = *iox2::container::StaticString<SOURCE_STRING_SIZE>::from_utf8("ABCD");
    constexpr uint64_t const DESTINATION_STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<DESTINATION_STRING_SIZE>::from_utf8("GHIJK");
    ASSERT_EQ(&(sut = test_string), &sut);
}


TEST(StaticString, try_push_back_appends_character_to_string_if_there_is_room) {
    constexpr uint64_t const STRING_SIZE = 5;
    iox2::container::StaticString<STRING_SIZE> sut;
    ASSERT_TRUE(sut.try_push_back('A'));
    ASSERT_EQ(sut.size(), 1);
    EXPECT_EQ(*sut.unchecked_code_units().back_element(), 'A');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "A");
    ASSERT_TRUE(sut.try_push_back('B'));
    ASSERT_EQ(sut.size(), 2);
    EXPECT_EQ(*sut.unchecked_code_units().back_element(), 'B');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "AB");
    ASSERT_TRUE(sut.try_push_back('C'));
    ASSERT_EQ(sut.size(), 3);
    EXPECT_EQ(*sut.unchecked_code_units().back_element(), 'C');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABC");
    ASSERT_TRUE(sut.try_push_back('D'));
    ASSERT_EQ(sut.size(), 4);
    EXPECT_EQ(*sut.unchecked_code_units().back_element(), 'D');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCD");
    ASSERT_TRUE(sut.try_push_back('E'));
    ASSERT_EQ(sut.size(), 5);
    EXPECT_EQ(*sut.unchecked_code_units().back_element(), 'E');
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABCDE");
}

TEST(StaticString, try_push_back_fails_if_there_is_no_room) {
    constexpr uint64_t const STRING_SIZE = 3;
    iox2::container::StaticString<STRING_SIZE> sut;
    ASSERT_TRUE(sut.try_push_back('A') && sut.try_push_back('B') && sut.try_push_back('C'));
    ASSERT_EQ(sut.size(), sut.capacity());
    EXPECT_FALSE(sut.try_push_back('D'));
    EXPECT_STREQ(sut.unchecked_access().c_str(), "ABC");
}

TEST(StaticString, static_string_with_capacity_0_can_never_be_pushed_into) {
    iox2::container::StaticString<0> sut;
    ASSERT_TRUE(sut.empty());
    ASSERT_EQ(sut.size(), 0);
    ASSERT_FALSE(sut.try_push_back('A'));
    ASSERT_STREQ(sut.unchecked_access().c_str(), "");
}

TEST(StaticString, try_pop_removes_last_element_from_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked("ABCDE");
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABCDE");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 4);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABCD");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 3);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABC");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 2);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "AB");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 1);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "A");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 0);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "");
}

TEST(StaticString, try_pop_fails_on_empty_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8_null_terminated_unchecked("A");
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_TRUE(sut.empty());
    ASSERT_FALSE(sut.try_pop_back());
    ASSERT_TRUE(sut.empty());
    ASSERT_FALSE(sut.try_pop_back());
    ASSERT_TRUE(sut.empty());
}

TEST(StaticString, size_returns_number_of_elements_in_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    iox2::container::StaticString<STRING_SIZE> sut;
    ASSERT_EQ(sut.size(), 0);
    ASSERT_TRUE(sut.try_push_back('A'));
    ASSERT_EQ(sut.size(), 1);
    ASSERT_TRUE(sut.try_push_back('A'));
    ASSERT_EQ(sut.size(), 2);
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 1);
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_EQ(sut.size(), 0);
}

TEST(StaticString, empty_indicates_whether_the_string_is_empty) {
    constexpr uint64_t const STRING_SIZE = 5;
    iox2::container::StaticString<STRING_SIZE> sut;
    ASSERT_TRUE(sut.empty());
    ASSERT_TRUE(sut.try_push_back('A'));
    ASSERT_TRUE(!sut.empty());
    ASSERT_TRUE(sut.try_push_back('A'));
    ASSERT_TRUE(!sut.empty());
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_TRUE(!sut.empty());
    ASSERT_TRUE(sut.try_pop_back());
    ASSERT_TRUE(sut.empty());
}

TEST(StaticString, code_unit_element_at_accesses_element_by_index) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.code_units().element_at(0));
    ASSERT_EQ(sut.code_units().element_at(0).value(), 'A');
    ASSERT_TRUE(sut.code_units().element_at(1));
    ASSERT_EQ(sut.code_units().element_at(1).value(), 'B');
    ASSERT_TRUE(sut.code_units().element_at(2));
    ASSERT_EQ(sut.code_units().element_at(2).value(), 'C');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("x");
    ASSERT_TRUE(sut.code_units().element_at(0));
    ASSERT_EQ(sut.code_units().element_at(0).value(), 'x');
}

TEST(StaticString, code_unit_element_at_returns_nullopt_if_index_out_of_bounds) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_FALSE(sut.code_units().element_at(4));
    ASSERT_FALSE(sut.code_units().element_at(5));
    ASSERT_FALSE(sut.code_units().element_at(9999));
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("x");
    ASSERT_FALSE(sut.code_units().element_at(1));
    ASSERT_FALSE(sut.code_units().element_at(2));
    sut = iox2::container::StaticString<STRING_SIZE>();
    ASSERT_FALSE(sut.code_units().element_at(0));
}

TEST(StaticString, code_unit_back_element_returns_last_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.code_units().back_element());
    ASSERT_EQ(sut.code_units().back_element().value(), 'C');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("XYZ");
    ASSERT_TRUE(sut.code_units().back_element());
    ASSERT_EQ(sut.code_units().back_element().value(), 'Z');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("P");
    ASSERT_TRUE(sut.code_units().back_element());
    ASSERT_EQ(sut.code_units().back_element().value(), 'P');
}

TEST(StaticString, code_unit_back_element_returns_nullopt_on_empty_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    iox2::container::StaticString<STRING_SIZE> const sut;
    ASSERT_FALSE(sut.code_units().back_element());
}

TEST(StaticString, code_unit_front_element_returns_first_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.code_units().front_element());
    ASSERT_EQ(sut.code_units().front_element().value(), 'A');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("XYZ");
    ASSERT_TRUE(sut.code_units().front_element());
    ASSERT_EQ(sut.code_units().front_element().value(), 'X');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("P");
    ASSERT_TRUE(sut.code_units().front_element());
    ASSERT_EQ(sut.code_units().front_element().value(), 'P');
}

TEST(StaticString, unchecked_code_unit_element_at_accesses_element_by_index) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.unchecked_code_units().element_at(0));
    ASSERT_EQ(sut.unchecked_code_units().element_at(0).value(), 'A');
    ASSERT_TRUE(sut.unchecked_code_units().element_at(1));
    ASSERT_EQ(sut.unchecked_code_units().element_at(1).value(), 'B');
    ASSERT_TRUE(sut.unchecked_code_units().element_at(2));
    ASSERT_EQ(sut.unchecked_code_units().element_at(2).value(), 'C');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("x");
    ASSERT_TRUE(sut.unchecked_code_units().element_at(0));
    ASSERT_EQ(sut.unchecked_code_units().element_at(0).value(), 'x');
}

TEST(StaticString, unchecked_code_unit_element_at_allows_modification_of_indexed_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.unchecked_code_units().element_at(0));
    sut.unchecked_code_units().element_at(0).value().get() = 'X';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XBC");
    ASSERT_TRUE(sut.unchecked_code_units().element_at(1));
    sut.unchecked_code_units().element_at(1).value().get() = 'Y';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XYC");
    ASSERT_TRUE(sut.unchecked_code_units().element_at(2));
    sut.unchecked_code_units().element_at(2).value().get() = 'Z';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XYZ");
}

TEST(StaticString, unchecked_code_unit_element_at_returns_nullopt_if_index_out_of_bounds) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_FALSE(sut.unchecked_code_units().element_at(4));
    ASSERT_FALSE(sut.unchecked_code_units().element_at(5));
    ASSERT_FALSE(sut.unchecked_code_units().element_at(9999));
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("x");
    ASSERT_FALSE(sut.unchecked_code_units().element_at(1));
    ASSERT_FALSE(sut.unchecked_code_units().element_at(2));
    sut = iox2::container::StaticString<STRING_SIZE>();
    ASSERT_FALSE(sut.unchecked_code_units().element_at(0));
}

TEST(StaticString, unchecked_code_unit_back_element_returns_last_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.unchecked_code_units().back_element());
    ASSERT_EQ(sut.unchecked_code_units().back_element().value(), 'C');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("XYZ");
    ASSERT_TRUE(sut.unchecked_code_units().back_element());
    ASSERT_EQ(sut.unchecked_code_units().back_element().value(), 'Z');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("P");
    ASSERT_TRUE(sut.unchecked_code_units().back_element());
    ASSERT_EQ(sut.unchecked_code_units().back_element().value(), 'P');
}

TEST(StaticString, unchecked_code_unit_back_element_allows_modification_of_last_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.unchecked_code_units().back_element());
    sut.unchecked_code_units().back_element().value().get() = 'Z';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABZ");
}

TEST(StaticString, unchecked_code_unit_back_element_returns_nullopt_on_empty_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    iox2::container::StaticString<STRING_SIZE> sut;
    ASSERT_FALSE(sut.unchecked_code_units().back_element());
}

TEST(StaticString, unchecked_code_unit_front_element_returns_first_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.unchecked_code_units().front_element());
    ASSERT_EQ(sut.unchecked_code_units().front_element().value(), 'A');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("XYZ");
    ASSERT_TRUE(sut.unchecked_code_units().front_element());
    ASSERT_EQ(sut.unchecked_code_units().front_element().value(), 'X');
    sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("P");
    ASSERT_TRUE(sut.unchecked_code_units().front_element());
    ASSERT_EQ(sut.unchecked_code_units().front_element().value(), 'P');
}

TEST(StaticString, unchecked_code_unit_front_element_allows_modification_of_first_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_TRUE(sut.unchecked_code_units().front_element());
    sut.unchecked_code_units().front_element().value().get() = '0';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "0BC");
}

TEST(StaticString, unchecked_code_unit_front_element_returns_nullopt_on_empty_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    iox2::container::StaticString<STRING_SIZE> sut;
    ASSERT_FALSE(sut.unchecked_code_units().front_element());
}

TEST(StaticString, unchecked_const_subscript_operator_allows_accessing_chars_by_index) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    ASSERT_EQ(sut.unchecked_access()[0], 'A');
    ASSERT_EQ(sut.unchecked_access()[1], 'B');
    ASSERT_EQ(sut.unchecked_access()[2], 'C');
    ASSERT_EQ(sut.unchecked_access()[3], 'D');
    ASSERT_EQ(sut.unchecked_access()[4], '\0');
    auto const sut2 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("XY");
    ASSERT_EQ(sut2.unchecked_access()[0], 'X');
    ASSERT_EQ(sut2.unchecked_access()[1], 'Y');
    ASSERT_EQ(sut2.unchecked_access()[2], '\0');
    ASSERT_EQ(sut2.unchecked_access()[3], '\0');
    ASSERT_EQ(sut2.unchecked_access()[4], '\0');
}

TEST(StaticString, unchecked_subscript_operator_allows_accessing_chars_by_index) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    sut.unchecked_access()[0] = 'X';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XBC");
    sut.unchecked_access()[1] = 'Y';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XYC");
    sut.unchecked_access()[2] = 'Z';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XYZ");
}

TEST(StaticString, unchecked_const_begin_returns_pointer_to_first_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(sut.unchecked_access().begin(), &sut.unchecked_access()[0]);
}

TEST(StaticString, unchecked_begin_returns_mutable_pointer_to_first_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(sut.unchecked_access().begin(), &sut.unchecked_access()[0]);
    *sut.unchecked_access().begin() = 'X';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XBC");
}

TEST(StaticString, unchecked_const_end_returns_pointer_to_one_past_last_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_EQ(sut.unchecked_access().end(), &sut.unchecked_access()[sut.size()]);
}

TEST(StaticString, unchecked_end_returns_mutable_pointer_to_one_past_last_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_EQ(sut.unchecked_access().end(), &sut.unchecked_access()[sut.size()]);
    *(sut.unchecked_access().end() - 1) = 'X';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABX");
}

TEST(StaticString, unchecked_const_data_returns_pointer_to_first_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(sut.unchecked_access().data(), &sut.unchecked_access()[0]);
}

TEST(StaticString, unchecked_data_returns_mutable_pointer_to_first_element) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(sut.unchecked_access().data(), &sut.unchecked_access()[0]);
    sut.unchecked_access().data()[0] = 'X';
    ASSERT_STREQ(sut.unchecked_access().c_str(), "XBC");
}

TEST(StaticString, unchecked_const_c_str_returns_pointer_to_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABC");
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(sut.unchecked_access().c_str(), &sut.unchecked_access()[0]);
}

TEST(StaticString, unchecked_c_str_returns_pointer_to_string) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto sut = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    iox2::container::testing::opaque_use(sut);
    ASSERT_STREQ(sut.unchecked_access().c_str(), "ABC");
    // NOLINTNEXTLINE(readability-container-data-pointer) testing
    ASSERT_EQ(sut.unchecked_access().c_str(), &sut.unchecked_access()[0]);
}

TEST(StaticString, equality_operator_checks_for_string_equality) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const sut1 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    auto const sut2 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    EXPECT_EQ(sut1, sut2);
    auto const sut3 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    EXPECT_FALSE(sut1 == sut3);
    auto const sut4 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCDE");
    EXPECT_FALSE(sut1 == sut4);
    auto const sut5 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("");
    EXPECT_FALSE(sut1 == sut5);
    // NOLINTNEXTLINE(readability-container-size-empty) testing
    EXPECT_EQ(sut5, iox2::container::StaticString<STRING_SIZE>());
}

TEST(StaticString, not_equal_operator_checks_for_string_inequality) {
    constexpr uint64_t const STRING_SIZE = 5;
    auto const sut1 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    auto const sut2 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCD");
    EXPECT_FALSE(sut1 != sut2);
    auto const sut3 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABC");
    EXPECT_NE(sut1, sut3);
    auto const sut4 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("ABCDE");
    EXPECT_NE(sut1, sut4);
    auto const sut5 = *iox2::container::StaticString<STRING_SIZE>::from_utf8("");
    EXPECT_NE(sut1, sut5);
    // NOLINTNEXTLINE(readability-container-size-empty) testing
    EXPECT_FALSE(sut5 != iox2::container::StaticString<STRING_SIZE>());
}


} // namespace
