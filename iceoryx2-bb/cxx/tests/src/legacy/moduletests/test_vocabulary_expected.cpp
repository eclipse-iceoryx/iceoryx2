// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox2/legacy/detail/hoofs_error_reporting.hpp"
#include "iox2/legacy/expected.hpp"

#include "iox2/legacy/testing/fatal_failure.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;
using namespace ::iox2::legacy;
using namespace ::iox2::legacy::testing;

namespace {
class MockCallables {
  public:
    MockCallables() = default;
    MOCK_METHOD0(onSuccess, void());
    MOCK_METHOD0(onEmpty, void());
    MOCK_METHOD0(onError, void());
};


struct TestClass {
    // the parameters can be swapped without changing the outcome, methods which
    // work on them are commutative
    // NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
    TestClass(int a, int b)
        : m_a(a)
        , m_b(b) {
    }

    // should be non const to be usable in tests
    // NOLINTNEXTLINE(readability-make-member-function-const)
    int gimme() {
        return m_a + m_b;
    }

    int constGimme() const {
        return m_a + m_b;
    }

    bool operator==(const TestClass& rhs) const {
        return (m_a == rhs.m_a) && (m_b == rhs.m_b);
    }

    int m_a;
    int m_b;
};

struct NonTrivialTestClass {
    // swapped parameters will be directly detected with failing tests
    // NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
    NonTrivialTestClass(int a, int b)
        : m_a(a)
        , m_b(b) {
    }

    NonTrivialTestClass(const NonTrivialTestClass& other) {
        *this = other;
    }
    NonTrivialTestClass(NonTrivialTestClass&& other) noexcept {
        *this = std::move(other);
    }

    NonTrivialTestClass& operator=(const NonTrivialTestClass& rhs) {
        if (this != &rhs) {
            m_a = rhs.m_a;
            m_b = rhs.m_b;
            m_moved = rhs.m_moved;
        }
        return *this;
    }

    NonTrivialTestClass& operator=(NonTrivialTestClass&& rhs) noexcept {
        if (this != &rhs) {
            m_a = rhs.m_a;
            m_b = rhs.m_b;
            m_moved = rhs.m_moved;
            rhs.m_moved = true;
        }
        return *this;
    }

    int m_a { 0 };
    int m_b { 0 };
    bool m_moved { false };
};

struct ClassWithMoveCtorAndNoMoveAssignment {
    ClassWithMoveCtorAndNoMoveAssignment() = default;
    ClassWithMoveCtorAndNoMoveAssignment(const ClassWithMoveCtorAndNoMoveAssignment&) = delete;
    ClassWithMoveCtorAndNoMoveAssignment(ClassWithMoveCtorAndNoMoveAssignment&&) = default;
    ~ClassWithMoveCtorAndNoMoveAssignment() = default;

    ClassWithMoveCtorAndNoMoveAssignment& operator=(const ClassWithMoveCtorAndNoMoveAssignment&) = delete;
    ClassWithMoveCtorAndNoMoveAssignment& operator=(ClassWithMoveCtorAndNoMoveAssignment&&) = delete;
};

class expected_test : public Test { };

enum class TestError : uint8_t {
    ERROR1,
    ERROR2,
    ERROR3
};

TEST_F(expected_test, CreateWithPODTypeIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "5b91db8c-5d2e-44a4-8cac-4ee436b5fe8e");
    constexpr int VALUE = 123;
    auto sut = expected<int, TestError>(in_place, VALUE);
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value(), Eq(VALUE));
}

TEST_F(expected_test, CreateWithVoidTypeIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "5baee3cb-4f81-4245-b9f9-d733d14d6d4a");
    auto sut = expected<void, TestError>(in_place);
    ASSERT_THAT(sut.has_value(), Eq(true));
}

TEST_F(expected_test, CreateWithErrorResultsInError) {
    ::testing::Test::RecordProperty("TEST_ID", "a2d10c89-6fc8-4c08-9e2d-9f61988ebb3f");
    auto sut = expected<int, TestError>(unexpect, TestError::ERROR1);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR1));
}

TEST_F(expected_test, ConstCreateWithErrorResultsInError) {
    ::testing::Test::RecordProperty("TEST_ID", "581447a6-0705-494b-8159-cf3434080a06");
    const auto sut = expected<int, TestError>(unexpect, TestError::ERROR2);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR2));
}

TEST_F(expected_test, ErrorTypeOnlyCreateWithErrorResultsInError) {
    ::testing::Test::RecordProperty("TEST_ID", "b01b2217-e67a-4bbf-b1a8-95d9b348d66e");
    auto sut = expected<void, TestError>(unexpect, TestError::ERROR1);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR1));
}

TEST_F(expected_test, CreateFromConstErrorResultsInError) {
    ::testing::Test::RecordProperty("TEST_ID", "8e4324ad-f221-4038-91ad-61a1567545dd");
    auto constError = err(TestError::ERROR3);
    auto sut = expected<int, TestError>(constError);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR3));
}

TEST_F(expected_test, CreateFromConstSuccessResultsInCorrectValue) {
    ::testing::Test::RecordProperty("TEST_ID", "cb20f217-6617-4c9e-8185-35cbf2bb8f3e");
    constexpr int VALUE = 424242;
    auto constSuccess = ok(VALUE);
    auto sut = expected<int, TestError>(constSuccess);
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value(), Eq(VALUE));
}

TEST_F(expected_test, CreateWithComplexTypeIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "508a39f7-905a-4d9a-a61b-43145e546eca");
    constexpr int VALUE_A = 12;
    constexpr int VALUE_B = 222;
    auto sut = expected<TestClass, TestError>(in_place, VALUE_A, VALUE_B);
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.value().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateWithSTLTypeIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "24fddc69-64ca-4b69-baab-a58293657cac");
    const std::string ERROR_VALUE = "RedAlert";
    auto sut = expected<int, std::string>(unexpect, ERROR_VALUE);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(ERROR_VALUE));
}

TEST_F(expected_test, CreateWithComplexErrorResultsInError) {
    ::testing::Test::RecordProperty("TEST_ID", "71e6ea31-d6e3-42a0-a63d-4bbd39c7341c");
    constexpr int VALUE_A = 313;
    constexpr int VALUE_B = 212;
    auto sut = expected<int, TestClass>(unexpect, VALUE_A, VALUE_B);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.error().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateRValueAndGetErrorResultsInCorrectError) {
    ::testing::Test::RecordProperty("TEST_ID", "b032400a-cd08-4ae7-af0c-5ae0362b4dc0");
    constexpr int VALUE_A = 131;
    constexpr int VALUE_B = 121;
    auto sut = expected<int, TestClass>(unexpect, VALUE_A, VALUE_B).error();
    EXPECT_THAT(sut.m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateConstRValueAndGetErrorResultsInCorrectError) {
    ::testing::Test::RecordProperty("TEST_ID", "936bb9c0-2559-4716-ba03-d5b927fff40f");
    constexpr int VALUE_A = 123;
    constexpr int VALUE_B = 122;
    using SutType = expected<int, TestClass>;
    auto sut = static_cast<const SutType&&>(SutType(unexpect, VALUE_A, VALUE_B)).error();
    EXPECT_THAT(sut.m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateLValueAndGetErrorResultsInCorrectError) {
    ::testing::Test::RecordProperty("TEST_ID", "a167d79e-9c50-45d8-afb8-5a4cc2f3da1b");
    constexpr int VALUE_A = 133;
    constexpr int VALUE_B = 112;
    auto sut = expected<int, TestClass>(unexpect, VALUE_A, VALUE_B);
    EXPECT_THAT(sut.error().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.error().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, ConstCreateLValueAndGetErrorResultsInCorrectError) {
    ::testing::Test::RecordProperty("TEST_ID", "e56063ea-8b7c-4d47-a898-fe609ea3b283");
    constexpr int VALUE_A = 112;
    constexpr int VALUE_B = 211;
    const auto sut = expected<int, TestClass>(unexpect, VALUE_A, VALUE_B);
    EXPECT_THAT(sut.error().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.error().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateRValueAndGetValueResultsInCorrectValue) {
    ::testing::Test::RecordProperty("TEST_ID", "fb5a3954-50de-419a-b29d-635d068fcb84");
    constexpr int VALUE_A = 141;
    constexpr int VALUE_B = 131;
    auto sut = expected<TestClass, TestError>(in_place, VALUE_A, VALUE_B).value();
    EXPECT_THAT(sut.m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateConstRValueAndGetValueResultsInCorrectValue) {
    ::testing::Test::RecordProperty("TEST_ID", "4af92b14-3b70-4ddd-8589-991abe3c8571");
    constexpr int VALUE_A = 144;
    constexpr int VALUE_B = 113;
    using SutType = expected<TestClass, TestError>;
    auto sut = static_cast<const SutType&&>(SutType(in_place, VALUE_A, VALUE_B)).value();
    EXPECT_THAT(sut.m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateLValueAndGetValueResultsInCorrectValue) {
    ::testing::Test::RecordProperty("TEST_ID", "5adabab2-3329-47bf-bfb7-fe8aa98eacc2");
    constexpr int VALUE_A = 114;
    constexpr int VALUE_B = 311;
    auto sut = expected<TestClass, TestError>(in_place, VALUE_A, VALUE_B);
    EXPECT_THAT(sut.value().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.value().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, ConstCreateLValueAndGetValueResultsInCorrectValue) {
    ::testing::Test::RecordProperty("TEST_ID", "e33c2d23-7914-4ba7-a8ee-37e3c91c4a74");
    constexpr int VALUE_A = 411;
    constexpr int VALUE_B = 133;
    const auto sut = expected<TestClass, TestError>(in_place, VALUE_A, VALUE_B);
    EXPECT_THAT(sut.value().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.value().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, CreateWithValueAndMoveCtorLeadsToMovedSource) {
    ::testing::Test::RecordProperty("TEST_ID", "8da72983-3046-4dde-8de5-5eed89de0ccf");
    constexpr int A { 177 };
    constexpr int B { 188 };
    auto sutSource = expected<NonTrivialTestClass, int>(in_place, A, B);
    auto sutDestination { std::move(sutSource) };

    // NOLINTJUSTIFICATION we explicitly want to test the defined state of a moved expected
    // NOLINTBEGIN(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutSource.has_value());
    EXPECT_TRUE(sutSource.value().m_moved);
    // NOLINTEND(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutDestination.has_value());
    EXPECT_FALSE(sutDestination.value().m_moved);
    EXPECT_EQ(sutDestination.value().m_a, A);
    EXPECT_EQ(sutDestination.value().m_b, B);
}

TEST_F(expected_test, CreateWithErrorAndMoveCtorLeadsToMovedSource) {
    ::testing::Test::RecordProperty("TEST_ID", "d7784813-458b-40f3-b6db-01521e57175e");
    constexpr int A { 22 };
    constexpr int B { 33 };
    auto sutSource = expected<int, NonTrivialTestClass>(unexpect, A, B);
    auto sutDestination { std::move(sutSource) };

    // NOLINTJUSTIFICATION we explicitly want to test the defined state of a moved expected
    // NOLINTBEGIN(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutSource.has_error());
    EXPECT_TRUE(sutSource.error().m_moved);
    // NOLINTEND(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutDestination.has_error());
    EXPECT_FALSE(sutDestination.error().m_moved);
    EXPECT_EQ(sutDestination.error().m_a, A);
    EXPECT_EQ(sutDestination.error().m_b, B);
}

TEST_F(expected_test, CreateWithValueAndMoveAssignmentLeadsToMovedSource) {
    ::testing::Test::RecordProperty("TEST_ID", "eb5f326b-8446-4914-bdca-8d6ba20103fe");
    constexpr int A { 73 };
    constexpr int B { 37 };
    auto sutSource = expected<NonTrivialTestClass, int>(in_place, A, B);
    auto sutDestination = std::move(sutSource);

    // NOLINTJUSTIFICATION we explicitly want to test the defined state of a moved expected
    // NOLINTBEGIN(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutSource.has_value());
    EXPECT_TRUE(sutSource.value().m_moved);
    // NOLINTEND(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutDestination.has_value());
    EXPECT_FALSE(sutDestination.value().m_moved);
    EXPECT_EQ(sutDestination.value().m_a, A);
    EXPECT_EQ(sutDestination.value().m_b, B);
}

TEST_F(expected_test, CreateWithErrorAndMoveAssignmentLeadsToMovedSource) {
    ::testing::Test::RecordProperty("TEST_ID", "ef2a799d-982e-447d-8f93-f7ad63c091e0");
    constexpr int A { 44 };
    constexpr int B { 55 };
    auto sutSource = expected<int, NonTrivialTestClass>(unexpect, A, B);
    auto sutDestination = std::move(sutSource);

    // NOLINTJUSTIFICATION we explicitly want to test the defined state of a moved expected
    // NOLINTBEGIN(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutSource.has_error());
    EXPECT_TRUE(sutSource.error().m_moved);
    // NOLINTEND(bugprone-use-after-move,hicpp-invalid-access-moved,clang-analyzer-cplusplus.Move)
    ASSERT_TRUE(sutDestination.has_error());
    EXPECT_FALSE(sutDestination.error().m_moved);
    EXPECT_EQ(sutDestination.error().m_a, A);
    EXPECT_EQ(sutDestination.error().m_b, B);
}

TEST_F(expected_test, CreateWithOkFreeFunctionWithVoidValueTypeIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "6d582b25-1c7d-4519-837c-55d151b324ff");
    expected<void, TestError> sut = ok();
    ASSERT_THAT(sut.has_value(), Eq(true));
}

TEST_F(expected_test, CreateWithOkFreeFunctionByCopyIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "d3c24c27-432d-4a4b-8d55-6e723bc88c46");
    constexpr int VALUE = 111;
    expected<int, TestError> sut = ok(VALUE);
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value(), Eq(VALUE));
}

TEST_F(expected_test, CreateWithOkFreeFunctionByMoveIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "b1320e1f-3613-4085-8125-fc95d584681c");
    constexpr int A { 44 };
    constexpr int B { 55 };
    NonTrivialTestClass value { A, B };
    expected<NonTrivialTestClass, TestError> sut = ok(std::move(value));
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value().m_a, Eq(A));
    EXPECT_THAT(sut.value().m_b, Eq(B));
}

TEST_F(expected_test, CreateWithOkFreeFunctionByForwardingIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "a3d41181-f4ad-4431-9441-7dfaeb8d6f7f");
    constexpr int A { 44 };
    constexpr int B { 55 };
    expected<NonTrivialTestClass, TestError> sut = ok<NonTrivialTestClass>(A, B);
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value().m_a, Eq(A));
    EXPECT_THAT(sut.value().m_b, Eq(B));
}

TEST_F(expected_test, CreateWithErrFreeFunctionByCopyIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "bb641919-e319-4e9c-af67-e1e8d5dab682");
    expected<int, TestError> sut = err(TestError::ERROR1);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR1));
}

TEST_F(expected_test, CreateWithErrFreeFunctionByMoveIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "f99af97a-16b2-41e6-a808-2d58bfe0fc57");
    constexpr int A { 666 };
    constexpr int B { 73 };
    NonTrivialTestClass error { A, B };
    expected<int, NonTrivialTestClass> sut = err(std::move(error));
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error().m_a, Eq(A));
    EXPECT_THAT(sut.error().m_b, Eq(B));
}

TEST_F(expected_test, CreateWithErrFreeFunctionByForwardingIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "08411afa-e1d3-4a28-9680-f89796f86340");
    constexpr int A { 44 };
    constexpr int B { 55 };
    expected<int, NonTrivialTestClass> sut = err<NonTrivialTestClass>(A, B);
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error().m_a, Eq(A));
    EXPECT_THAT(sut.error().m_b, Eq(B));
}

TEST_F(expected_test, CopyConstructorWorksWithValueContent) {
    ::testing::Test::RecordProperty("TEST_ID", "71ce4717-bf77-47ea-8ed9-3a890b13ce88");
    constexpr int VALUE { 455171 };

    expected<int, NonTrivialTestClass> sut = ok(VALUE);
    expected<int, NonTrivialTestClass> sut_copy(sut);

    ASSERT_THAT(sut.has_value(), Eq(true));
    ASSERT_THAT(sut_copy.has_value(), Eq(true));

    ASSERT_THAT(sut.value(), Eq(VALUE));
    ASSERT_THAT(sut_copy.value(), Eq(VALUE));
}

TEST_F(expected_test, CopyConstructorWorksWithErrorContent) {
    ::testing::Test::RecordProperty("TEST_ID", "e0be66c3-05fc-4030-92d0-8ad84111e86f");
    constexpr int A { 719122 };
    constexpr int B { 700012 };

    expected<int, NonTrivialTestClass> sut = err<NonTrivialTestClass>(A, B);
    expected<int, NonTrivialTestClass> sut_copy(sut);

    ASSERT_THAT(sut.has_error(), Eq(true));
    ASSERT_THAT(sut_copy.has_error(), Eq(true));

    ASSERT_THAT(sut.error().m_a, Eq(A));
    ASSERT_THAT(sut.error().m_b, Eq(B));
    ASSERT_THAT(sut_copy.error().m_a, Eq(A));
    ASSERT_THAT(sut_copy.error().m_b, Eq(B));
}

TEST_F(expected_test, MoveConstructorWorksWithValueContent) {
    ::testing::Test::RecordProperty("TEST_ID", "8f188c06-6675-4e9b-bd72-28ea813cb149");
    constexpr int VALUE { 919155171 };

    expected<int, NonTrivialTestClass> sut = ok(VALUE);
    expected<int, NonTrivialTestClass> sut_move(std::move(sut));

    ASSERT_THAT(sut_move.has_value(), Eq(true));
    ASSERT_THAT(sut_move.value(), Eq(VALUE));
}

TEST_F(expected_test, MoveConstructorWorksWithErrorContent) {
    ::testing::Test::RecordProperty("TEST_ID", "c8eb14d0-fee4-474b-ab9a-33e834a47f19");
    constexpr int A { 7331 };
    constexpr int B { 73391 };

    expected<int, NonTrivialTestClass> sut = err<NonTrivialTestClass>(A, B);
    expected<int, NonTrivialTestClass> sut_move(std::move(sut));

    ASSERT_THAT(sut_move.has_error(), Eq(true));
    ASSERT_THAT(sut_move.error().m_a, Eq(A));
    ASSERT_THAT(sut_move.error().m_b, Eq(B));
}

TEST_F(expected_test, CopyAssignmentWorksWithValueContent) {
    ::testing::Test::RecordProperty("TEST_ID", "e16679e7-91cb-4e3c-869a-bdca338c4963");
    constexpr int VALUE { 333195171 };

    expected<int, NonTrivialTestClass> sut = ok(VALUE);
    expected<int, NonTrivialTestClass> sut_copy = err<NonTrivialTestClass>(1, 2);

    sut_copy = sut;

    ASSERT_THAT(sut.has_value(), Eq(true));
    ASSERT_THAT(sut_copy.has_value(), Eq(true));

    ASSERT_THAT(sut.value(), Eq(VALUE));
    ASSERT_THAT(sut_copy.value(), Eq(VALUE));
}

TEST_F(expected_test, CopyAssignmentWorksWithErrorContent) {
    ::testing::Test::RecordProperty("TEST_ID", "66db5dea-8543-4ad0-9705-1c23ed316463");
    constexpr int A { 557331 };
    constexpr int B { 5573391 };

    expected<int, NonTrivialTestClass> sut = err<NonTrivialTestClass>(A, B);
    expected<int, NonTrivialTestClass> sut_copy = ok(1231);

    sut_copy = sut;

    ASSERT_THAT(sut.has_error(), Eq(true));
    ASSERT_THAT(sut_copy.has_error(), Eq(true));

    ASSERT_THAT(sut.error().m_a, Eq(A));
    ASSERT_THAT(sut.error().m_b, Eq(B));
    ASSERT_THAT(sut_copy.error().m_a, Eq(A));
    ASSERT_THAT(sut_copy.error().m_b, Eq(B));
}

TEST_F(expected_test, MoveAssignmentWorksWithValueContent) {
    ::testing::Test::RecordProperty("TEST_ID", "87ca60fe-7b29-4144-91fe-80ebfed644bd");
    constexpr int VALUE { 910001 };

    expected<int, NonTrivialTestClass> sut = ok(VALUE);
    expected<int, NonTrivialTestClass> sut_move = err<NonTrivialTestClass>(1, 2);

    sut_move = std::move(sut);

    ASSERT_THAT(sut_move.has_value(), Eq(true));
    ASSERT_THAT(sut_move.value(), Eq(VALUE));
}

TEST_F(expected_test, MoveAssignmentWorksWithErrorContent) {
    ::testing::Test::RecordProperty("TEST_ID", "82691fe2-fd18-4b43-b926-e9e67699760e");
    constexpr int A { 9557431 };
    constexpr int B { 95574391 };

    expected<int, NonTrivialTestClass> sut = err<NonTrivialTestClass>(A, B);
    expected<int, NonTrivialTestClass> sut_move = ok(121);

    sut_move = sut;

    ASSERT_THAT(sut_move.has_error(), Eq(true));
    ASSERT_THAT(sut_move.error().m_a, Eq(A));
    ASSERT_THAT(sut_move.error().m_b, Eq(B));
}

TEST_F(expected_test, BoolOperatorReturnsError) {
    ::testing::Test::RecordProperty("TEST_ID", "f1e30651-a0e9-4c73-b2bf-57f36fc7eddf");
    constexpr int VALUE_A = 55899;
    constexpr int VALUE_B = 11;
    expected<int, TestClass> sut = err<TestClass>(VALUE_A, VALUE_B);
    ASSERT_THAT(sut.operator bool(), Eq(false));
    EXPECT_THAT(sut.error().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.error().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, BoolOperatorReturnsNoError) {
    ::testing::Test::RecordProperty("TEST_ID", "aec3e2a3-b7ae-4778-ac1d-d52e64b9b2d3");
    constexpr int VALUE_A = 5599;
    constexpr int VALUE_B = 8111;
    expected<TestClass, TestError> sut = ok<TestClass>(VALUE_A, VALUE_B);

    ASSERT_THAT(sut.operator bool(), Eq(true));
    EXPECT_THAT(sut.value().m_a, Eq(VALUE_A));
    EXPECT_THAT(sut.value().m_b, Eq(VALUE_B));
}

TEST_F(expected_test, ErrorTypeOnlyBoolOperatorReturnsError) {
    ::testing::Test::RecordProperty("TEST_ID", "7949f68f-c21c-43f1-ad8d-dc51eeee3257");
    expected<void, TestError> sut = err(TestError::ERROR1);
    ASSERT_THAT(sut.operator bool(), Eq(false));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR1));
}

TEST_F(expected_test, ErrorTypeOnlyBoolOperatorReturnsNoError) {
    ::testing::Test::RecordProperty("TEST_ID", "4585b1bf-cd6f-44ac-8409-75dc14fa252a");
    expected<void, TestError> sut = ok();
    ASSERT_THAT(sut.operator bool(), Eq(true));
}

TEST_F(expected_test, HasValueIsTrueWhenHasErrorIsFalse) {
    ::testing::Test::RecordProperty("TEST_ID", "cf339ae0-bc54-4584-bef1-9471eb2d5370");
    expected<void, TestError> sut = ok();
    ASSERT_TRUE(sut.has_value());
    ASSERT_FALSE(sut.has_error());
}

TEST_F(expected_test, HasValueIsFalseWhenHasErrorIsTrue) {
    ::testing::Test::RecordProperty("TEST_ID", "28f6a33a-5264-4507-a6e3-879a297dc1e5");
    expected<void, TestError> sut = err(TestError::ERROR1);
    ASSERT_FALSE(sut.has_value());
    ASSERT_TRUE(sut.has_error());
}

TEST_F(expected_test, ArrowOperatorWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "39898e81-d4ad-4f27-8c45-d29c80114be2");
    constexpr int VALUE_A = 55;
    constexpr int VALUE_B = 81;
    expected<TestClass, TestError> sut = ok<TestClass>(VALUE_A, VALUE_B);
    ASSERT_THAT(sut.has_error(), Eq(false));
    EXPECT_THAT(sut->gimme(), Eq(VALUE_A + VALUE_B));
}

TEST_F(expected_test, ConstArrowOperatorWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "b35a05e9-6dbc-4cfb-94c2-85ca9d214bb4");
    constexpr int VALUE_A = 554;
    constexpr int VALUE_B = 811;
    const expected<TestClass, TestError> sut(ok<TestClass>(VALUE_A, VALUE_B));
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut->constGimme(), Eq(VALUE_A + VALUE_B));
}

TEST_F(expected_test, DereferencingOperatorWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "11ddbd46-3a2f-43cd-a2d2-ebe2ad4019db");
    constexpr int VALUE = 1652;
    expected<int, TestError> sut = ok(VALUE);
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(*sut, Eq(VALUE));
}

TEST_F(expected_test, ConstDereferencingOperatorWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "f09b9476-a4f6-4f56-9692-3c00146410fd");
    const expected<int, TestError> sut(ok(981));
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(*sut, Eq(981));
}

TEST_F(expected_test, CreateFromInPlaceTypeLeadsToValidVoidValueTypeSut) {
    ::testing::Test::RecordProperty("TEST_ID", "91a8ad7f-4843-4bd9-a56b-0561ae6b56cb");
    expected<void, TestError> sut { in_place };
    ASSERT_THAT(sut.has_value(), Eq(true));
}

TEST_F(expected_test, CreateFromInPlaceTypeLeadsToValidSut) {
    ::testing::Test::RecordProperty("TEST_ID", "3a527c62-aaea-44ae-9b99-027c19d032b5");
    constexpr int VALUE = 42;
    expected<int, TestError> sut { in_place, VALUE };
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value(), Eq(VALUE));
}

TEST_F(expected_test, CreateFromUnexpectTypeLeadsToValidSutWithError) {
    ::testing::Test::RecordProperty("TEST_ID", "20ddbfc0-2235-46c3-9618-dd75e9d3c699");
    expected<int, TestError> sut { unexpect, TestError::ERROR3 };
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR3));
}

TEST_F(expected_test, CreateFromEmptySuccessTypeLeadsToValidSut) {
    ::testing::Test::RecordProperty("TEST_ID", "0204f08f-fb6d-45bb-aac7-fd14152ab1bf");
    expected<void, TestError> sut { ok() };
    ASSERT_THAT(sut.has_error(), Eq(false));
}

TEST_F(expected_test, CreateFromSuccessTypeLeadsToValidSut) {
    ::testing::Test::RecordProperty("TEST_ID", "fb83b62e-4e17-480b-8425-72181e6dd55d");
    constexpr int VALUE = 55;
    expected<int, TestError> sut { ok(VALUE) };
    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut.value(), Eq(VALUE));
}

TEST_F(expected_test, CreateFromErrorLeadsToCorrectError) {
    ::testing::Test::RecordProperty("TEST_ID", "cb7e783d-0a79-45ce-9ea7-3b6e28631ceb");
    expected<int, TestError> sut { err(TestError::ERROR2) };
    ASSERT_THAT(sut.has_error(), Eq(true));
    EXPECT_THAT(sut.error(), Eq(TestError::ERROR2));
}

TEST_F(expected_test, ConvertNonEmptySuccessResultToVoidValueTypeResultIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "b14f4aaa-abd0-4b99-84df-d644506712fa");
    constexpr int VALUE = 91823;
    expected<int, TestError> sut { ok(VALUE) };
    expected<void, TestError> sut2 = sut;
    EXPECT_THAT(sut2.has_value(), Eq(true));
}

TEST_F(expected_test, ConvertConstNonEmptySuccessResultToVoidValueTypeResultIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "6ccaf1cf-1b09-4930-ad33-8f961aca4c2e");
    const expected<int, TestError> sut { ok(123) };
    expected<void, TestError> sut2 = sut;
    EXPECT_THAT(sut2.has_value(), Eq(true));
}

TEST_F(expected_test, ConvertNonEmptyErrorResultVoidValueTypeResultIsSuccessful) {
    ::testing::Test::RecordProperty("TEST_ID", "5907d318-cf1a-46f1-9016-07096153d7d9");
    expected<int, TestError> sut { err(TestError::ERROR2) };
    expected<void, TestError> sut2 = sut;
    EXPECT_THAT(sut2.has_error(), Eq(true));
    EXPECT_THAT(sut2.error(), Eq(TestError::ERROR2));
}

TEST_F(expected_test, MoveAssignmentIsNotEnforcedInMoveConstructor) {
    ::testing::Test::RecordProperty("TEST_ID", "71cd336f-798b-4f08-9ab6-be3c429c1674");
    {
        auto sut = expected<ClassWithMoveCtorAndNoMoveAssignment, int>(in_place);
        /// this should compile, if not then we enforce move assignment hidden in the implementation
        expected<ClassWithMoveCtorAndNoMoveAssignment, int> destination { std::move(sut) };
        ASSERT_THAT(destination.has_value(), Eq(true));
    }

    /// same test with the void value type
    {
        auto sut = expected<void, ClassWithMoveCtorAndNoMoveAssignment>(unexpect);
        /// this should compile, if not then we enforce move assignment hidden in the implementation
        expected<void, ClassWithMoveCtorAndNoMoveAssignment> destination { std::move(sut) };
        ASSERT_THAT(destination.has_error(), Eq(true));
    }
}

TEST_F(expected_test, AccessingValueOfLValueExpectedWhichContainsErrorWithArrowOpLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "1a821c6f-83db-4fe1-8adf-873afa1251a1");

#if defined _WIN32
    GTEST_SKIP() << "The 'AccessingValueOfLValueExpectedWhichContainsErrorWithArrowOpLeadsToErrorHandlerCall' test is "
                    "disabled on Windows";
#else
    expected<TestClass, TestError> sut = err(TestError::ERROR1);

    IOX2_EXPECT_FATAL_FAILURE([&] { IOX2_DISCARD_RESULT(sut->m_a); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingValueOfConstLValueExpectedWhichContainsErrorWithArrowOpLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "c4f04d7c-9fa3-48f6-a6fd-b8e4e47b7632");

#if defined _WIN32
    GTEST_SKIP() << "The 'AccessingValueOfConstLValueExpectedWhichContainsErrorWithArrowOpLeadsToErrorHandlerCall' "
                    "test is disabled on Windows";
#else
    const expected<TestClass, TestError> sut = err(TestError::ERROR1);

    IOX2_EXPECT_FATAL_FAILURE([&] { IOX2_DISCARD_RESULT(sut->m_a); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingValueOfLValueExpectedWhichContainsErrorWithDerefOpLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "08ce6a3f-3813-46de-8e1e-3ffe8087521e");

#if defined _WIN32
    GTEST_SKIP() << "The 'AccessingValueOfLValueExpectedWhichContainsErrorWithDerefOpLeadsToErrorHandlerCall' test is "
                    "disabled on Windows";
#else
    expected<TestClass, TestError> sut = err(TestError::ERROR1);

    IOX2_EXPECT_FATAL_FAILURE([&] { *sut; }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingValueOfConstLValueExpectedWhichContainsErrorWithDerefOpLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "838dd364-f91f-40a7-9720-2b662a045b1e");

#if defined _WIN32
    GTEST_SKIP() << "The 'AccessingValueOfConstLValueExpectedWhichContainsErrorWithDerefOpLeadsToErrorHandlerCall' "
                    "test is disabled on Windows";
#else
    const expected<TestClass, TestError> sut = err(TestError::ERROR1);

    IOX2_EXPECT_FATAL_FAILURE([&] { *sut; }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingValueOfLValueExpectedWhichContainsErrorLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "92139583-b8d6-4d83-ae7e-f4109b98d214");

#if defined _WIN32
    GTEST_SKIP()
        << "The 'AccessingValueOfLValueExpectedWhichContainsErrorLeadsToErrorHandlerCall' test is disabled on Windows";
#else
    expected<TestClass, TestError> sut = err(TestError::ERROR1);

    IOX2_EXPECT_FATAL_FAILURE([&] { sut.value(); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingValueOfConstLValueExpectedWhichContainsErrorLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "1bcbb835-8b4c-4430-a534-a26573c2380d");

#if defined _WIN32
    GTEST_SKIP() << "The 'AccessingValueOfConstLValueExpectedWhichContainsErrorLeadsToErrorHandlerCall' test is "
                    "disabled on Windows";
#else
    const expected<TestClass, TestError> sut = err(TestError::ERROR1);

    IOX2_EXPECT_FATAL_FAILURE([&] { sut.value(); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingValueOfRValueExpectedWhichContainsErrorLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "32d59b52-81f5-417a-8670-dfb2c54fedfb");

#if defined _WIN32
    GTEST_SKIP()
        << "The 'AccessingValueOfRValueExpectedWhichContainsErrorLeadsToErrorHandlerCall' test is disabled on Windows";
#else
    expected<TestClass, TestError> sut = err(TestError::ERROR1);

    IOX2_EXPECT_FATAL_FAILURE([&] { std::move(sut).value(); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingErrorOfLValueExpectedWhichContainsValueLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "aee85ead-e066-49fd-99fe-6f1a6045756d");

#if defined _WIN32
    GTEST_SKIP()
        << "The 'AccessingErrorOfLValueExpectedWhichContainsValueLeadsToErrorHandlerCall' test is disabled on Windows";
#else
    constexpr int VALID_VALUE { 42 };
    expected<TestClass, TestError> sut = ok<TestClass>(VALID_VALUE, VALID_VALUE);

    IOX2_EXPECT_FATAL_FAILURE([&] { sut.error(); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingErrorOfConstLValueExpectedWhichContainsValueLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "a49cf02e-b165-4fd6-9c24-65cedc6cddb9");

#if defined _WIN32
    GTEST_SKIP() << "The 'AccessingErrorOfConstLValueExpectedWhichContainsValueLeadsToErrorHandlerCall' test is "
                    "disabled on Windows";
#else
    constexpr int VALID_VALUE { 42 };
    const expected<TestClass, TestError> sut = ok<TestClass>(VALID_VALUE, VALID_VALUE);

    IOX2_EXPECT_FATAL_FAILURE([&] { sut.error(); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, AccessingErrorOfRValueExpectedWhichContainsValueLeadsToErrorHandlerCall) {
    ::testing::Test::RecordProperty("TEST_ID", "0ea90b5d-1af6-494a-b35c-da103bed2331");

#if defined _WIN32
    GTEST_SKIP()
        << "The 'AccessingErrorOfRValueExpectedWhichContainsValueLeadsToErrorHandlerCall' test is disabled on Windows";
#else
    constexpr int VALID_VALUE { 42 };
    expected<TestClass, TestError> sut = ok<TestClass>(VALID_VALUE, VALID_VALUE);

    IOX2_EXPECT_FATAL_FAILURE([&] { std::move(sut).error(); }, iox2::legacy::er::ENFORCE_VIOLATION);
#endif
}

TEST_F(expected_test, TwoVoidValueTypeExpectedWithEqualErrorAreEqual) {
    ::testing::Test::RecordProperty("TEST_ID", "471b406d-8dd3-4b82-9d46-00c21d257461");
    expected<void, TestError> sut1 = err(TestError::ERROR1);
    expected<void, TestError> sut2 = err(TestError::ERROR1);

    EXPECT_TRUE(sut1 == sut2);
    EXPECT_FALSE(sut1 != sut2);
}

TEST_F(expected_test, TwoVoidValueTypeExpectedWithUnequalErrorAreUnequal) {
    ::testing::Test::RecordProperty("TEST_ID", "bcc2f9f1-72a1-41ed-ac8a-2f48cdcfbc56");
    expected<void, TestError> sut1 = err(TestError::ERROR1);
    expected<void, TestError> sut2 = err(TestError::ERROR2);

    EXPECT_FALSE(sut1 == sut2);
    EXPECT_TRUE(sut1 != sut2);
}

TEST_F(expected_test, TwoVoidValueTypeExpectedWithValuesAreEqual) {
    ::testing::Test::RecordProperty("TEST_ID", "75b25c16-fb79-4589-ab0f-bc73bb9fc2bb");
    expected<void, TestError> sut1 = ok();
    expected<void, TestError> sut2 = ok();

    EXPECT_TRUE(sut1 == sut2);
    EXPECT_FALSE(sut1 != sut2);
}

TEST_F(expected_test, TwoVoidValueTypeExpectedWithErrorAndValueAreUnequal) {
    ::testing::Test::RecordProperty("TEST_ID", "2108715f-e71c-4778-bb64-553996e860b4");
    expected<void, TestError> sut1 = err(TestError::ERROR1);
    expected<void, TestError> sut2 = ok();

    EXPECT_FALSE(sut1 == sut2);
    EXPECT_TRUE(sut1 != sut2);
}

TEST_F(expected_test, TwoExpectedWithEqualErrorAreEqual) {
    ::testing::Test::RecordProperty("TEST_ID", "b1a3b106-06f2-4667-ac25-7a9d9689c219");
    expected<TestClass, TestError> sut1 = err(TestError::ERROR1);
    expected<TestClass, TestError> sut2 = err(TestError::ERROR1);

    EXPECT_TRUE(sut1 == sut2);
    EXPECT_FALSE(sut1 != sut2);
}

TEST_F(expected_test, TwoExpectedsWithUnequalErrorAreUnequal) {
    ::testing::Test::RecordProperty("TEST_ID", "25250c6b-aa8f-40ad-ace9-2c55ce8eeaa2");
    expected<TestClass, TestError> sut1 = err(TestError::ERROR1);
    expected<TestClass, TestError> sut2 = err(TestError::ERROR2);

    EXPECT_FALSE(sut1 == sut2);
    EXPECT_TRUE(sut1 != sut2);
}

TEST_F(expected_test, TwoExpectedWithEqualValueAreEqual) {
    ::testing::Test::RecordProperty("TEST_ID", "278c2fd5-2b48-49d1-a8a4-8ca52b99de41");
    constexpr int VAL_1 { 42 };
    constexpr int VAL_2 { 73 };
    expected<TestClass, TestError> sut1 = ok<TestClass>(VAL_1, VAL_2);
    expected<TestClass, TestError> sut2 = ok<TestClass>(VAL_1, VAL_2);

    EXPECT_TRUE(sut1 == sut2);
    EXPECT_FALSE(sut1 != sut2);
}

TEST_F(expected_test, TwoExpectedWithUnequalValueAreUnequal) {
    ::testing::Test::RecordProperty("TEST_ID", "5f6a8760-6fdf-4ab8-a7d5-d751390aa672");
    constexpr int VAL_1 { 42 };
    constexpr int VAL_2 { 73 };
    expected<TestClass, TestError> sut1 = ok<TestClass>(VAL_1, VAL_1);
    expected<TestClass, TestError> sut2 = ok<TestClass>(VAL_2, VAL_2);

    EXPECT_FALSE(sut1 == sut2);
    EXPECT_TRUE(sut1 != sut2);
}

TEST_F(expected_test, TwoExpectedWithErrorAndValueAreUnequal) {
    ::testing::Test::RecordProperty("TEST_ID", "aa912753-09af-46d5-92d5-52cad69795ad");
    constexpr int VAL { 42 };
    expected<TestClass, TestError> sut1 = err(TestError::ERROR1);
    expected<TestClass, TestError> sut2 = ok<TestClass>(VAL, VAL);

    EXPECT_FALSE(sut1 == sut2);
    EXPECT_TRUE(sut1 != sut2);
}
} // namespace
