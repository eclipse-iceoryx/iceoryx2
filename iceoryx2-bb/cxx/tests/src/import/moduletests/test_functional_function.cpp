// Copyright (c) 2020 - 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox/bump_allocator.hpp"
#include "iox/function.hpp"
#include "test.hpp"

#include <functional>
#include <iostream>

using namespace ::testing;
using namespace iox;

namespace
{

constexpr uint64_t Bytes = 128U;

using signature = int32_t(int32_t);
template <typename T>
using fixed_size_function = function<T, Bytes>;
using test_function = fixed_size_function<signature>;


// helper template to count construction and copy statistics,
// for our purpose (test_function) we do not need to distinguish
// between copy (move) ctor and copy(move) assignment
template <typename T>
class Counter
{
  public:
    static uint64_t numCreated;
    static uint64_t numCopied;
    static uint64_t numMoved;
    static uint64_t numDestroyed;

    Counter()
    {
        ++numCreated;
    }

    Counter(const Counter& rhs IOX_MAYBE_UNUSED)
    {
        ++numCreated;
        ++numCopied;
    }

    Counter(Counter&& rhs IOX_MAYBE_UNUSED) noexcept
    {
        ++numMoved;
    }

    ~Counter()
    {
        ++numDestroyed;
    }

    Counter& operator=(const Counter& rhs)
    {
        if (this != &rhs)
        {
            ++numCopied;
        }
        return *this;
    }

    Counter& operator=(Counter&& rhs) noexcept
    {
        if (this != &rhs)
        {
            ++numMoved;
        }
        return *this;
    }

    static void resetCounts()
    {
        numCreated = 0U;
        numCopied = 0U;
        numMoved = 0U;
        numDestroyed = 0U;
    }
};

template <typename T>
uint64_t Counter<T>::numCreated = 0U;

template <typename T>
uint64_t Counter<T>::numCopied = 0U;

template <typename T>
uint64_t Counter<T>::numMoved = 0U;

template <typename T>
uint64_t Counter<T>::numDestroyed = 0U;


class Functor : public Counter<Functor>
{
  public:
    explicit Functor(int32_t state)
        : m_state(state)
    {
    }

    int32_t operator()(int32_t n)
    {
        m_state += n;
        return m_state;
    }

    // integer arg to satisfy signature requirement of our test_function
    int32_t getState(int32_t n = 0) const
    {
        return m_state + n;
    }

    int32_t m_state{0};
};

int32_t freeFunction(int32_t n)
{
    return n + 1;
}

struct Arg : Counter<Arg>
{
    Arg() = default;
    explicit Arg(int32_t value)
        : value(value) {};
    Arg(const Arg&) = default;
    Arg& operator=(const Arg&) = default;

    // We cannot delete the move ctor, the function wrapper requires the arguments to be copy-constructible.
    // According to the standard this means the copy Ctor must exist and move cannot be explicitly deleted.
    // Move does not necessarily have to be defined, in which case the compiler will perform a copy
    // whenever a move would be possible.
    // Note that this is mainly an issue if the argument is passed by value.
    // The std::function also fails to compile in this case (gcc implementation).

    int32_t value{0};
};

int32_t freeFunctionWithCopyableArg(const Arg& arg)
{
    return arg.value;
}


class function_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }

    static int32_t staticFunction(int32_t n)
    {
        return n + 1;
    }
};

TEST_F(function_test, ConstructionFromFunctorIsCallable)
{
    ::testing::Test::RecordProperty("TEST_ID", "2969913d-a849-47c5-a76b-f32481e03ea5");
    Functor f(73);
    Functor::resetCounts();
    test_function sut(f);

    EXPECT_EQ(Functor::numCreated, 1U);
    EXPECT_EQ(sut(1), f(1));
}

TEST_F(function_test, ConstructionFromLambdaIsCallable)
{
    ::testing::Test::RecordProperty("TEST_ID", "f42a8511-b78d-47b5-aa7f-ae227ae12465");
    int32_t capture = 37;
    auto lambda = [state = capture](int32_t n) { return state + n; };
    test_function sut(lambda);

    EXPECT_EQ(sut(1), lambda(1));
}

TEST_F(function_test, ConstructionFromFreeFunctionIsCallable)
{
    ::testing::Test::RecordProperty("TEST_ID", "2d808b65-182b-44b0-a501-c9b6ab3c80e7");
    test_function sut(freeFunction);

    EXPECT_EQ(sut(1), freeFunction(1));
}

TEST_F(function_test, ConstructionFromStaticFunctionIsCallable)
{
    ::testing::Test::RecordProperty("TEST_ID", "24f95326-9d93-4ce1-8338-3582d9a82af3");
    // is essentially also a free function but we test the case to be sure
    test_function sut(staticFunction);

    EXPECT_EQ(sut(1), staticFunction(1));
}

TEST_F(function_test, ConstructionFromMemberFunctionIsCallable)
{
    ::testing::Test::RecordProperty("TEST_ID", "ac4311a5-8e85-4051-92cc-ca28e679c5ab");
    Functor f(37);
    test_function sut(f, &Functor::operator());

    auto result = f(1);
    EXPECT_EQ(sut(1), result + 1);
}

TEST_F(function_test, ConstructionFromConstMemberFunctionIsCallable)
{
    ::testing::Test::RecordProperty("TEST_ID", "a59e5060-ebca-42dd-ae04-0bacab7c3805");
    Functor f(37);
    test_function sut(f, &Functor::getState);

    auto state = f.getState(1);
    EXPECT_EQ(sut(1), state);
    EXPECT_EQ(f.getState(1), state); // state is unchanged by the previous call
}

TEST_F(function_test, ConstructionFromAnotherFunctionIsCallable)
{
    ::testing::Test::RecordProperty("TEST_ID", "18e62771-8ed3-43eb-ba1d-876f3825e09e");
    constexpr int32_t INITIAL = 37;
    int32_t capture = INITIAL;
    auto lambda = [&](int32_t n) { return ++capture + n; };
    function<signature, Bytes / 2> f(lambda); // the other function type must be small enough to fit
    test_function sut(f);

    auto result = f(1);
    EXPECT_EQ(sut(1), result + 1);
    EXPECT_EQ(capture, INITIAL + 2);
}

TEST_F(function_test, FunctionStateIsIndependentOfSource)
{
    ::testing::Test::RecordProperty("TEST_ID", "8302046f-cd6a-4527-aca6-3e6408f87a6b");
    constexpr uint32_t INITIAL_STATE = 73;
    constexpr uint32_t MEMORY_SIZE = 1024;

    // NOLINTNEXTLINE(hicpp-no-malloc, cppcoreguidelines-no-malloc) low-level memory management for testing purpose
    void* memory = malloc(MEMORY_SIZE);
    iox::BumpAllocator allocator(memory, MEMORY_SIZE);
    auto allocationResult = allocator.allocate(sizeof(Functor), alignof(Functor));
    ASSERT_FALSE(allocationResult.has_error());

    auto* p = static_cast<Functor*>(allocationResult.value());
    p = new (p) Functor(INITIAL_STATE);

    // call the dtor in any case (even if the test fails due to ASSERT)
    std::unique_ptr<Functor, void (*)(Functor*)> guard(p, [](Functor* f) { f->~Functor(); });

    auto& functor = *p;

    // test whether the function really owns the functor
    // (no dependency or side effects)
    test_function sut(functor);

    // both increment their state independently
    EXPECT_EQ(sut(1U), functor(1U));

    guard.reset(); // call the deleter
    // NOLINTNEXTLINE(hicpp-no-malloc, cppcoreguidelines-no-malloc) low-level memory management for testing purpose
    free(memory);

    EXPECT_EQ(sut(1U), INITIAL_STATE + 2U);
}

// The implementation uses type erasure and we need to verify that the corresponding
// constructors and operators of the underlying object (functor) are called.

TEST_F(function_test, DestructorCallsDestructorOfStoredFunctor)
{
    ::testing::Test::RecordProperty("TEST_ID", "2481cf93-c63b-40b0-b6de-2213507efe33");
    Functor f(73);
    Functor::resetCounts();

    {
        test_function sut(f);
    }

    EXPECT_EQ(Functor::numDestroyed, 1U);
}

TEST_F(function_test, CopyCtorCopiesStoredFunctor)
{
    ::testing::Test::RecordProperty("TEST_ID", "e34fba7e-0c11-4535-8ac3-7d1b034fc793");
    Functor functor(73);
    test_function f(functor);
    Functor::resetCounts();

    // NOLINTJUSTIFICATION the copy constructor is tested here
    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
    test_function sut(f);

    EXPECT_EQ(Functor::numCopied, 1U);
    EXPECT_EQ(sut(1), f(1));
}

TEST_F(function_test, MoveCtorMovesStoredFunctor)
{
    ::testing::Test::RecordProperty("TEST_ID", "0b9d8b1e-81a6-4242-8f31-aadf2a6c0f91");
    Functor functor(73);
    test_function f(functor);
    Functor::resetCounts();

    test_function sut(std::move(f));

    EXPECT_EQ(Functor::numMoved, 1U);
    EXPECT_EQ(sut(1), functor(1));
}

TEST_F(function_test, CopyAssignmentCopiesStoredFunctor)
{
    ::testing::Test::RecordProperty("TEST_ID", "8ef88318-0aa0-4766-8b3c-a9cc197f88fd");
    test_function f(Functor(73));
    test_function sut(Functor(42));

    Functor::resetCounts();
    sut = f;

    EXPECT_EQ(Functor::numDestroyed, 1U);
    EXPECT_EQ(Functor::numCopied, 1U);
    EXPECT_EQ(sut(1), f(1));
}

TEST_F(function_test, MoveAssignmentMovesStoredFunctor)
{
    ::testing::Test::RecordProperty("TEST_ID", "684f7c51-5532-46d1-91ea-7e7e7e76534b");
    Functor functor(73);
    test_function f(functor);
    test_function sut(Functor(42));

    Functor::resetCounts();
    sut = std::move(f);

    // destroy previous Functor in sut and Functor in f after move
    // (f is not callable but can be reassigned)
    EXPECT_EQ(Functor::numDestroyed, 2U);
    EXPECT_EQ(Functor::numMoved, 1U);
    EXPECT_EQ(sut(1), functor(1));
}


TEST_F(function_test, CopyCtorCopiesStoredFreeFunction)
{
    ::testing::Test::RecordProperty("TEST_ID", "8f95a82a-c879-48b1-aa56-316bf15b983a");
    test_function f(freeFunction);
    // NOLINTJUSTIFICATION the copy constructor is tested here
    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
    test_function sut(f);

    EXPECT_EQ(sut(1), f(1));
}

TEST_F(function_test, MoveCtorMovesStoredFreeFunction)
{
    ::testing::Test::RecordProperty("TEST_ID", "efcd5ae0-393f-4243-8825-871f7f59a9c0");
    test_function f(freeFunction);
    test_function sut(std::move(f));

    EXPECT_EQ(sut(1), freeFunction(1));
}

TEST_F(function_test, CopyAssignmentCopiesStoredFreeFunction)
{
    ::testing::Test::RecordProperty("TEST_ID", "29ebca31-0266-4741-84b3-b3cbecfc7b4a");
    test_function f(freeFunction);
    test_function sut(Functor(73));

    Functor::resetCounts();
    sut = f;

    EXPECT_EQ(Functor::numDestroyed, 1U);
    EXPECT_EQ(Functor::numCopied, 0U);
    EXPECT_EQ(Functor::numMoved, 0U);
    EXPECT_EQ(sut(1), f(1));
}

TEST_F(function_test, MoveAssignmentMovesStoredFreeFunction)
{
    ::testing::Test::RecordProperty("TEST_ID", "414ec34a-f5e3-4ab6-bfab-60796bbd7b8a");
    test_function f(freeFunction);
    test_function sut(Functor(73));

    Functor::resetCounts();
    sut = std::move(f);

    EXPECT_EQ(Functor::numDestroyed, 1U);
    EXPECT_EQ(Functor::numCopied, 0U);
    EXPECT_EQ(Functor::numMoved, 0U);
    EXPECT_EQ(sut(1), freeFunction(1));
}

TEST_F(function_test, MemberSwapWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "85ba9d33-f552-4aa9-a214-24464a5ca934");
    Functor f1(73);
    Functor f2(37);
    test_function sut1(f1);
    test_function sut2(f2);

    sut1.swap(sut2);

    EXPECT_EQ(sut1(1), f2(1));
    EXPECT_EQ(sut2(1), f1(1));
}

TEST_F(function_test, StaticSwapWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "0b27cb60-85ae-4942-b448-1f9b00a253fa");
    Functor f1(73);
    Functor f2(37);
    test_function sut1(f1);
    test_function sut2(f2);

    swap(sut1, sut2);

    EXPECT_EQ(sut1(1), f2(1));
    EXPECT_EQ(sut2(1), f1(1));
}

TEST_F(function_test, FunctorOfSizeSmallerThanStorageBytesCanBeStored)
{
    ::testing::Test::RecordProperty("TEST_ID", "34de556c-95f4-4d7b-b01b-377c08529f62");
    // it will not compile if the storage is too small,
    constexpr auto BYTES = test_function::required_storage_size<Functor>();
    EXPECT_LE(sizeof(Functor), BYTES);
    Functor f(73);
    function<signature, BYTES> sut(f);
}

TEST_F(function_test, IsStorableIsConsistent)
{
    ::testing::Test::RecordProperty("TEST_ID", "78fd4207-9ef4-459d-96f4-9cca98135b47");
    constexpr auto BYTES = test_function::required_storage_size<Functor>();
    constexpr auto RESULT = function<signature, BYTES>::is_storable<Functor>();
    EXPECT_TRUE(RESULT);
}

TEST_F(function_test, IsNotStorableDueToSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "4ecd7078-5b3d-4fd5-b5af-296401b652ce");
    constexpr auto BYTES = test_function::required_storage_size<Functor>();
    constexpr auto RESULT = function<signature, BYTES - alignof(Functor)>::is_storable<Functor>();
    EXPECT_FALSE(RESULT);
}

TEST_F(function_test, IsNotStorableDueToSignature)
{
    ::testing::Test::RecordProperty("TEST_ID", "a7a5e2a6-68dd-477a-8eb0-573e57c7a3ae");
    auto nonStorable = []() {};
    using NonStorable = decltype(nonStorable);
    constexpr auto BYTES = test_function::required_storage_size<NonStorable>();
    constexpr auto RESULT = function<signature, BYTES>::is_storable<NonStorable>();
    EXPECT_FALSE(RESULT);
}


TEST_F(function_test, CallWithCopyConstructibleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "20018d76-6255-407a-b3d3-77b6b480067d");
    function<int32_t(const Arg&), 1024> sut(freeFunctionWithCopyableArg);
    std::function<int32_t(const Arg&)> func(freeFunctionWithCopyableArg);
    Arg::resetCounts();

    Arg arg(73);

    auto result = sut(arg);

    EXPECT_EQ(result, freeFunctionWithCopyableArg(arg));
    EXPECT_EQ(result, func(arg));
    // note that by using the numCopies counter we can observe that the std::function call also performs 2 copies of arg
    // in this case
}

TEST_F(function_test, CallWithVoidSignatureWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "dcc54ea2-ce1a-4142-a141-df6d0bbe9707");
    const int32_t initial = 73;
    int value = initial;
    auto lambda = [&]() { ++value; };
    function<void(void), 128> sut(lambda);

    sut();

    EXPECT_EQ(value, initial + 1);
}

TEST_F(function_test, CallWithReferenceArgumentsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "ef3fe399-cf1c-4d28-b688-b50ac9c1fe3e");
    const int32_t initial = 73;
    Arg arg(initial);

    auto lambda = [](Arg& a) { ++a.value; };
    function<void(Arg&), 128> sut(lambda);

    sut(arg);

    EXPECT_EQ(arg.value, initial + 1);
}

TEST_F(function_test, CallWithConstReferenceArgumentsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "80ea9066-918e-436d-9b99-11c6339412da");
    const int32_t initial = 73;
    Arg arg(initial);

    auto lambda = [](const Arg& a) { return a.value + 1; };
    function<int32_t(const Arg&), 128> sut(lambda);

    auto result = sut(arg);

    EXPECT_EQ(result, initial + 1);
}

TEST_F(function_test, CallWithValueArgumentsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b3ea6823-b392-418e-8be0-e8d69246e3c5");
    const int32_t initial = 73;
    Arg arg(initial);

    // NOLINTJUSTIFICATION value argument is tested here
    // NOLINTNEXTLINE(performance-unnecessary-value-param)
    auto lambda = [](const Arg a) { return a.value + 1; };
    function<int32_t(Arg&), 128> sut(lambda);

    auto result = sut(arg);

    EXPECT_EQ(result, initial + 1);
}

TEST_F(function_test, CallWithRValueReferenceArgumentsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "1c827680-a04d-4fca-bb22-96922d7192ab");
    const int32_t initial = 73;
    Arg arg(initial);

    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) this is okay for this test
    auto lambda = [](Arg&& a) { return a.value + 1; };
    function<int32_t(Arg&&), 128> sut(lambda);

    auto result = sut(std::move(arg));

    EXPECT_EQ(result, initial + 1);
}

TEST_F(function_test, CallWithMixedArgumentsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d26e380d-4b0e-4c9f-a1b9-c9e7ab3707e1");
    Arg arg1(1);
    Arg arg2(2);
    Arg arg3(3);
    Arg arg4(4);

    constexpr int32_t sum = 10;

    // NOLINTJUSTIFICATION value argument is tested here
    // NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved, performance-unnecessary-value-param)
    auto lambda = [](Arg& a1, const Arg& a2, Arg&& a3, Arg a4) { return a1.value + a2.value + a3.value + a4.value; };
    function<int32_t(Arg&, const Arg&, Arg&&, Arg), 128> sut(lambda);

    auto result = sut(arg1, arg2, std::move(arg3), arg4);

    EXPECT_EQ(result, sum);
}

} // namespace
