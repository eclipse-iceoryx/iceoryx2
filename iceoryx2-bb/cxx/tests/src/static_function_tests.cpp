// Copyright (c) 2020 - 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/legacy/uninitialized_array.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <functional>

using namespace ::testing;
using namespace iox2::bb;
using namespace iox2::legacy;

namespace {

constexpr uint64_t BUFFER_SIZE = 128U;

using Signature = int32_t(int32_t);
template <typename T>
using FixedSizeFunction = StaticFunction<T, BUFFER_SIZE>;
using TestFunction = FixedSizeFunction<Signature>;


// helper template to count construction and copy statistics,
// for our purpose (test_function) we do not need to distinguish
// between copy (move) ctor and copy(move) assignment
template <typename T>
class Counter {
  public:
    static uint64_t num_created;
    static uint64_t num_copied;
    static uint64_t num_moved;
    static uint64_t num_destroyed;

  private:
    friend T;

    Counter() {
        ++num_created;
    }

    Counter(const Counter& rhs IOX2_MAYBE_UNUSED) {
        ++num_created;
        ++num_copied;
    }

    Counter(Counter&& rhs IOX2_MAYBE_UNUSED) noexcept {
        ++num_moved;
    }

  public:
    ~Counter() {
        ++num_destroyed;
    }

    auto operator=(const Counter& rhs) -> Counter& {
        if (this != &rhs) {
            ++num_copied;
        }
        return *this;
    }

    auto operator=(Counter&& rhs) noexcept -> Counter& {
        if (this != &rhs) {
            ++num_moved;
        }
        return *this;
    }

    static auto reset_counts() -> void {
        num_created = 0U;
        num_copied = 0U;
        num_moved = 0U;
        num_destroyed = 0U;
    }
};

template <typename T>
uint64_t Counter<T>::num_created = 0U;

template <typename T>
uint64_t Counter<T>::num_copied = 0U;

template <typename T>
uint64_t Counter<T>::num_moved = 0U;

template <typename T>
uint64_t Counter<T>::num_destroyed = 0U;


class Functor : public Counter<Functor> {
  public:
    explicit Functor(int32_t state)
        : m_state(state) {
    }

    auto operator()(int32_t n) -> int32_t {
        m_state += n;
        return m_state;
    }

    // integer arg to satisfy signature requirement of our test_function
    auto get_state(int32_t n = 0) const -> int32_t {
        return m_state + n;
    }

  private:
    int32_t m_state { 0 };
};

auto free_function(int32_t n) -> int32_t {
    return n + 1;
}

struct Arg : Counter<Arg> {
    Arg() = default;
    explicit Arg(int32_t value)
        : value(value) { };

    // We cannot delete the move ctor, the function wrapper requires the arguments to be copy-constructible.
    // According to the standard this means the copy Ctor must exist and move cannot be explicitly deleted.
    // Move does not necessarily have to be defined, in which case the compiler will perform a copy
    // whenever a move would be possible.
    // Note that this is mainly an issue if the argument is passed by value.
    // The std::function also fails to compile in this case (gcc implementation).

    // NOLINTNEXTLINE(misc-non-private-member-variables-in-classes): just a helper struct
    int32_t value { 0 };
};

auto free_function_with_copyable_arg(const Arg& arg) -> int32_t {
    return arg.value;
}

class StaticFunctionFixture : public Test {
  public:
    void SetUp() override {
    }

    void TearDown() override {
    }

    static auto static_function(int32_t num) -> int32_t {
        return num + 1;
    }
};

TEST_F(StaticFunctionFixture, construction_from_functor_is_callable) {
    ::testing::Test::RecordProperty("TEST_ID", "2969913d-a849-47c5-a76b-f32481e03ea5");
    Functor func(73); // NOLINT: magic number
    Functor::reset_counts();
    const TestFunction sut(func);

    EXPECT_EQ(Functor::num_created, 1U);
    EXPECT_EQ(sut(1), func(1));
}

TEST_F(StaticFunctionFixture, construction_from_lambda_is_callable) {
    ::testing::Test::RecordProperty("TEST_ID", "f42a8511-b78d-47b5-aa7f-ae227ae12465");
    int32_t capture = 37; // NOLINT: magic number
    auto lambda = [state = capture](int32_t n) -> auto { return state + n; };
    const TestFunction sut(lambda);

    EXPECT_EQ(sut(1), lambda(1));
}

TEST_F(StaticFunctionFixture, construction_from_free_function_is_callable) {
    ::testing::Test::RecordProperty("TEST_ID", "2d808b65-182b-44b0-a501-c9b6ab3c80e7");
    const TestFunction sut(free_function);

    EXPECT_EQ(sut(1), free_function(1));
}

TEST_F(StaticFunctionFixture, construction_from_static_function_is_callable) {
    ::testing::Test::RecordProperty("TEST_ID", "24f95326-9d93-4ce1-8338-3582d9a82af3");
    // is essentially also a free function but we test the case to be sure
    const TestFunction sut(static_function);

    EXPECT_EQ(sut(1), static_function(1));
}

TEST_F(StaticFunctionFixture, construction_from_member_function_is_callable) {
    ::testing::Test::RecordProperty("TEST_ID", "ac4311a5-8e85-4051-92cc-ca28e679c5ab");
    Functor func(37); // NOLINT: magic number
    const TestFunction sut(func, &Functor::operator());

    auto result = func(1);
    EXPECT_EQ(sut(1), result + 1);
}

TEST_F(StaticFunctionFixture, construction_from_const_member_function_is_callable) {
    ::testing::Test::RecordProperty("TEST_ID", "a59e5060-ebca-42dd-ae04-0bacab7c3805");
    Functor func(37); // NOLINT: magic number
    const TestFunction sut(func, &Functor::get_state);

    auto state = func.get_state(1);
    EXPECT_EQ(sut(1), state);
    EXPECT_EQ(func.get_state(1), state); // state is unchanged by the previous call
}

TEST_F(StaticFunctionFixture, construction_from_another_function_is_callable) {
    ::testing::Test::RecordProperty("TEST_ID", "18e62771-8ed3-43eb-ba1d-876f3825e09e");
    constexpr int32_t INITIAL = 37;
    int32_t capture = INITIAL;
    auto lambda = [&](int32_t n) -> auto { return ++capture + n; };
    const StaticFunction<Signature, BUFFER_SIZE / 2> func(
        lambda); // the other function type must be small enough to fit
    const TestFunction sut(func);

    auto result = func(1);
    EXPECT_EQ(sut(1), result + 1);
    EXPECT_EQ(capture, INITIAL + 2);
}

TEST_F(StaticFunctionFixture, function_state_is_independent_of_source) {
    ::testing::Test::RecordProperty("TEST_ID", "8302046f-cd6a-4527-aca6-3e6408f87a6b");
    constexpr uint32_t INITIAL_STATE = 73;

    alignas(alignof(Functor)) UninitializedArray<char, sizeof(Functor)> memory;

    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory)
    auto* ptr = new (memory.begin()) Functor(INITIAL_STATE);

    // call the dtor in any case (even if the test fails due to ASSERT)
    std::unique_ptr<Functor, void (*)(Functor*)> guard(ptr, [](Functor* func) -> void { func->~Functor(); });

    auto& functor = *ptr;

    // test whether the function really owns the functor
    // (no dependency or side effects)
    const TestFunction sut(functor);

    // both increment their state independently
    EXPECT_EQ(sut(1U), functor(1U));

    guard.reset(); // call the deleter

    ptr->~Functor();

    EXPECT_EQ(sut(1U), INITIAL_STATE + 2U);
}

// The implementation uses type erasure and we need to verify that the corresponding
// constructors and operators of the underlying object (functor) are called.

TEST_F(StaticFunctionFixture, destructor_calls_destructor_of_stored_functor) {
    ::testing::Test::RecordProperty("TEST_ID", "2481cf93-c63b-40b0-b6de-2213507efe33");
    Functor func(73); // NOLINT: magic number
    Functor::reset_counts();

    {
        const TestFunction sut(func);
    }

    EXPECT_EQ(Functor::num_destroyed, 1U);
}

TEST_F(StaticFunctionFixture, copy_ctor_copies_stored_functor) {
    ::testing::Test::RecordProperty("TEST_ID", "e34fba7e-0c11-4535-8ac3-7d1b034fc793");
    Functor functor(73); // NOLINT: magic number
    const TestFunction func(functor);
    Functor::reset_counts();

    // NOLINTJUSTIFICATION the copy constructor is tested here
    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
    const TestFunction sut(func);

    EXPECT_EQ(Functor::num_copied, 1U);
    EXPECT_EQ(sut(1), func(1));
}

TEST_F(StaticFunctionFixture, move_ctor_moves_stored_functor) {
    ::testing::Test::RecordProperty("TEST_ID", "0b9d8b1e-81a6-4242-8f31-aadf2a6c0f91");
    Functor functor(73); // NOLINT: magic number
    TestFunction func(functor);
    Functor::reset_counts();

    const TestFunction sut(std::move(func));

    EXPECT_EQ(Functor::num_moved, 1U);
    EXPECT_EQ(sut(1), functor(1));
}

TEST_F(StaticFunctionFixture, copy_assignment_copies_stored_functor) {
    ::testing::Test::RecordProperty("TEST_ID", "8ef88318-0aa0-4766-8b3c-a9cc197f88fd");
    TestFunction func(Functor(73)); // NOLINT: magic number
    TestFunction sut(Functor(42));  // NOLINT: magic number

    Functor::reset_counts();
    sut = func;

    EXPECT_EQ(Functor::num_destroyed, 1U);
    EXPECT_EQ(Functor::num_copied, 1U);
    EXPECT_EQ(sut(1), func(1));
}

TEST_F(StaticFunctionFixture, move_assignment_moves_stored_functor) {
    ::testing::Test::RecordProperty("TEST_ID", "684f7c51-5532-46d1-91ea-7e7e7e76534b");
    Functor functor(73); // NOLINT: magic number
    TestFunction func(functor);
    TestFunction sut(Functor(42)); // NOLINT: magic number

    Functor::reset_counts();
    sut = std::move(func);

    // destroy previous Functor in sut and Functor in f after move
    // (f is not callable but can be reassigned)
    EXPECT_EQ(Functor::num_destroyed, 2U);
    EXPECT_EQ(Functor::num_moved, 1U);
    EXPECT_EQ(sut(1), functor(1));
}


TEST_F(StaticFunctionFixture, copy_ctor_copies_stored_free_function) {
    ::testing::Test::RecordProperty("TEST_ID", "8f95a82a-c879-48b1-aa56-316bf15b983a");
    const TestFunction func(free_function);
    // NOLINTJUSTIFICATION the copy constructor is tested here
    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
    const TestFunction sut(func);

    EXPECT_EQ(sut(1), func(1));
}

TEST_F(StaticFunctionFixture, move_ctor_moves_stored_free_function) {
    ::testing::Test::RecordProperty("TEST_ID", "efcd5ae0-393f-4243-8825-871f7f59a9c0");
    TestFunction func(free_function);
    const TestFunction sut(std::move(func));

    EXPECT_EQ(sut(1), free_function(1));
}

TEST_F(StaticFunctionFixture, copy_assignment_copies_stored_free_function) {
    ::testing::Test::RecordProperty("TEST_ID", "29ebca31-0266-4741-84b3-b3cbecfc7b4a");
    const TestFunction func(free_function);
    TestFunction sut(Functor(73)); // NOLINT: magic number

    Functor::reset_counts();
    sut = func;

    EXPECT_EQ(Functor::num_destroyed, 1U);
    EXPECT_EQ(Functor::num_copied, 0U);
    EXPECT_EQ(Functor::num_moved, 0U);
    EXPECT_EQ(sut(1), func(1));
}

TEST_F(StaticFunctionFixture, move_assignment_moves_stored_free_function) {
    ::testing::Test::RecordProperty("TEST_ID", "414ec34a-f5e3-4ab6-bfab-60796bbd7b8a");
    TestFunction func(free_function);
    TestFunction sut(Functor(73)); // NOLINT: magic number

    Functor::reset_counts();
    sut = std::move(func);

    EXPECT_EQ(Functor::num_destroyed, 1U);
    EXPECT_EQ(Functor::num_copied, 0U);
    EXPECT_EQ(Functor::num_moved, 0U);
    EXPECT_EQ(sut(1), free_function(1));
}

TEST_F(StaticFunctionFixture, member_swap_works) {
    ::testing::Test::RecordProperty("TEST_ID", "85ba9d33-f552-4aa9-a214-24464a5ca934");
    Functor func1(73); // NOLINT: magic number
    Functor func2(37); // NOLINT: magic number
    TestFunction sut1(func1);
    TestFunction sut2(func2);

    sut1.swap(sut2);

    EXPECT_EQ(sut1(1), func2(1));
    EXPECT_EQ(sut2(1), func1(1));
}

TEST_F(StaticFunctionFixture, static_swap_works) {
    ::testing::Test::RecordProperty("TEST_ID", "0b27cb60-85ae-4942-b448-1f9b00a253fa");
    Functor func1(73); // NOLINT: magic number
    Functor func2(37); // NOLINT: magic number
    TestFunction sut1(func1);
    TestFunction sut2(func2);

    swap(sut1, sut2);

    EXPECT_EQ(sut1(1), func2(1));
    EXPECT_EQ(sut2(1), func1(1));
}

TEST_F(StaticFunctionFixture, functor_of_size_smaller_than_storage_bytes_can_be_stored) {
    ::testing::Test::RecordProperty("TEST_ID", "34de556c-95f4-4d7b-b01b-377c08529f62");
    // it will not compile if the storage is too small,
    constexpr auto REQUIRED_SIZE = TestFunction::required_storage_size<Functor>();
    EXPECT_LE(sizeof(Functor), REQUIRED_SIZE);
    Functor func(73); // NOLINT: magic number
    const StaticFunction<Signature, REQUIRED_SIZE> sut(func);
}

TEST_F(StaticFunctionFixture, is_storable_is_consistent) {
    ::testing::Test::RecordProperty("TEST_ID", "78fd4207-9ef4-459d-96f4-9cca98135b47");
    constexpr auto REQUIRED_SIZE = TestFunction::required_storage_size<Functor>();
    constexpr auto RESULT = StaticFunction<Signature, REQUIRED_SIZE>::is_storable<Functor>();
    EXPECT_TRUE(RESULT);
}

TEST_F(StaticFunctionFixture, is_not_storable_due_to_size) {
    ::testing::Test::RecordProperty("TEST_ID", "4ecd7078-5b3d-4fd5-b5af-296401b652ce");
    constexpr auto REQUIRED_SIZE = TestFunction::required_storage_size<Functor>();
    constexpr auto RESULT = StaticFunction<Signature, REQUIRED_SIZE - alignof(Functor)>::is_storable<Functor>();
    EXPECT_FALSE(RESULT);
}

TEST_F(StaticFunctionFixture, is_not_storable_due_to_signature) {
    ::testing::Test::RecordProperty("TEST_ID", "a7a5e2a6-68dd-477a-8eb0-573e57c7a3ae");
    auto non_storable = []() -> auto { };
    using NonStorable = decltype(non_storable);
    constexpr auto REQUIRED_SIZE = TestFunction::required_storage_size<NonStorable>();
    constexpr auto RESULT = StaticFunction<Signature, REQUIRED_SIZE>::is_storable<NonStorable>();
    EXPECT_FALSE(RESULT);
}


TEST_F(StaticFunctionFixture, call_with_copy_constructible_argument) {
    ::testing::Test::RecordProperty("TEST_ID", "20018d76-6255-407a-b3d3-77b6b480067d");
    StaticFunction<int32_t(const Arg&), 1024> sut(free_function_with_copyable_arg); // NOLINT: magic number
    const std::function<int32_t(const Arg&)> func(free_function_with_copyable_arg);
    Arg::reset_counts();

    Arg arg(73); // NOLINT: magic number

    auto result = sut(arg);

    EXPECT_EQ(result, free_function_with_copyable_arg(arg));
    EXPECT_EQ(result, func(arg));
    // note that by using the numCopies counter we can observe that the std::function call also performs 2 copies of arg
    // in this case
}

TEST_F(StaticFunctionFixture, call_with_void_signature_works) {
    ::testing::Test::RecordProperty("TEST_ID", "dcc54ea2-ce1a-4142-a141-df6d0bbe9707");
    const int32_t initial = 73;
    int value = initial;
    auto lambda = [&]() -> auto { ++value; };
    StaticFunction<void(void), 128> sut(lambda); // NOLINT: magic number

    sut();

    EXPECT_EQ(value, initial + 1);
}

TEST_F(StaticFunctionFixture, call_with_reference_arguments_works) {
    ::testing::Test::RecordProperty("TEST_ID", "ef3fe399-cf1c-4d28-b688-b50ac9c1fe3e");
    const int32_t initial = 73;
    Arg arg(initial);

    auto lambda = [](Arg& arg) -> auto { ++arg.value; };
    StaticFunction<void(Arg&), 128> sut(lambda); // NOLINT: magic number

    sut(arg);

    EXPECT_EQ(arg.value, initial + 1);
}

TEST_F(StaticFunctionFixture, call_with_const_reference_arguments_works) {
    ::testing::Test::RecordProperty("TEST_ID", "80ea9066-918e-436d-9b99-11c6339412da");
    const int32_t initial = 73;
    const Arg arg(initial);

    auto lambda = [](const Arg& arg) -> auto { return arg.value + 1; };
    StaticFunction<int32_t(const Arg&), 128> sut(lambda); // NOLINT: magic number

    auto result = sut(arg);

    EXPECT_EQ(result, initial + 1);
}

TEST_F(StaticFunctionFixture, call_with_value_arguments_works) {
    ::testing::Test::RecordProperty("TEST_ID", "b3ea6823-b392-418e-8be0-e8d69246e3c5");
    const int32_t initial = 73;
    Arg arg(initial);

    // NOLINTJUSTIFICATION value argument is tested here
    // NOLINTNEXTLINE(performance-unnecessary-value-param)
    auto lambda = [](const Arg arg) -> auto { return arg.value + 1; };
    StaticFunction<int32_t(Arg&), 128> sut(lambda); // NOLINT: magic number

    auto result = sut(arg);

    EXPECT_EQ(result, initial + 1);
}

TEST_F(StaticFunctionFixture, call_with_r_value_reference_arguments_works) {
    ::testing::Test::RecordProperty("TEST_ID", "1c827680-a04d-4fca-bb22-96922d7192ab");
    const int32_t initial = 73;
    Arg arg(initial);

    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) this is okay for this test
    auto lambda = [](Arg&& arg) -> auto { return arg.value + 1; };
    StaticFunction<int32_t(Arg&&), 128> sut(lambda); // NOLINT: magic number

    auto result = sut(std::move(arg));

    EXPECT_EQ(result, initial + 1);
}

TEST_F(StaticFunctionFixture, call_with_mixed_arguments_works) {
    ::testing::Test::RecordProperty("TEST_ID", "d26e380d-4b0e-4c9f-a1b9-c9e7ab3707e1");
    Arg arg1(1);
    const Arg arg2(2);
    Arg arg3(3);
    const Arg arg4(4);

    constexpr int32_t SUM = 10;

    // NOLINTJUSTIFICATION value argument is tested here
    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved, performance-unnecessary-value-param)
    auto lambda = [](Arg& arg1, const Arg& arg2, Arg&& arg3, Arg arg4) -> auto {
        return arg1.value + arg2.value + arg3.value + arg4.value;
    };
    StaticFunction<int32_t(Arg&, const Arg&, Arg&&, Arg), 128> sut(lambda); // NOLINT: magic number

    auto result = sut(arg1, arg2, std::move(arg3), arg4);

    EXPECT_EQ(result, SUM);
}

} // namespace
