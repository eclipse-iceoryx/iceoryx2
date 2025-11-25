// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
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

#include "iceoryx_platform/atomic.hpp"

#include "test.hpp"

#include <atomic>

namespace
{
using namespace ::testing;

struct DummyStruct
{
    static constexpr uint64_t DEFAULT_VALUE{42};
    uint64_t value{DEFAULT_VALUE};
};

bool operator==(const DummyStruct& lhs, const DummyStruct& rhs)
{
    return lhs.value == rhs.value;
}

struct StdAtomic
{
    template <typename T>
    using SutType = std::atomic<T>;
};
struct IoxAtomic
{
    template <typename T>
    using SutType = iox::concurrent::Atomic<T>;
};

template <typename T>
class Atomic_test : public Test
{
  protected:
    template <typename U>
    using SutType = typename T::template SutType<U>;
};

using Implementations = Types<IoxAtomic, StdAtomic>;

TYPED_TEST_SUITE(Atomic_test, Implementations, );

#if __cplusplus >= 201703L
TYPED_TEST(Atomic_test, IsAlwaysLockFree)
{
    ::testing::Test::RecordProperty("TEST_ID", "28ac9afb-14a1-4970-b461-ec523f712d6b");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    EXPECT_THAT(SutInt::is_always_lock_free, Eq(true));
    EXPECT_THAT(SutFP::is_always_lock_free, Eq(true));
    EXPECT_THAT(SutPtr::is_always_lock_free, Eq(true));
    EXPECT_THAT(SutStruct::is_always_lock_free, Eq(true));
}
#endif

TYPED_TEST(Atomic_test, DefaultCtorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "840e5518-5bfa-4932-bdcf-125995dd7029");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    SutInt sut_int;
    SutFP sut_fp;
    SutPtr sut_ptr;
    SutStruct sut_struct;

    if (std::is_same<SutInt, std::atomic<uint64_t>>::value)
    {
        GTEST_SKIP() << "Default CTor test skipped for std::atomic since the iox::Atomic always initialized the value!";
    }

    EXPECT_THAT(sut_int.load(), Eq(uint64_t()));
    EXPECT_THAT(sut_fp.load(), Eq(double()));
    EXPECT_THAT(sut_ptr.load(), Eq(nullptr));
    EXPECT_THAT(sut_struct.load(), Eq(DummyStruct{DummyStruct::DEFAULT_VALUE}));
}

TYPED_TEST(Atomic_test, CtorArgumentWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "7dd9d9f8-4985-463d-af61-3b19b5a9cf34");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    constexpr uint64_t EXPECTED_INT{13};
    constexpr double EXPECTED_FP{73.37};
    constexpr DummyStruct EXPECTED_STRUCT{4242};
    uint64_t ptr_source{0};
    uint64_t* expected_ptr{&ptr_source};

    SutInt sut_int{EXPECTED_INT};
    volatile SutInt sut_volatile{EXPECTED_INT};
    SutFP sut_fp{EXPECTED_FP};
    SutPtr sut_ptr(expected_ptr);
    SutStruct sut_struct{EXPECTED_STRUCT};

    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_INT));
    EXPECT_THAT(sut_fp.load(), Eq(EXPECTED_FP));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_ptr));
    EXPECT_THAT(sut_struct.load(), Eq(EXPECTED_STRUCT));
}

TYPED_TEST(Atomic_test, AssignmentOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "2fc65a59-ff5b-4272-95ac-b7dce804f34e");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    constexpr uint64_t EXPECTED_INT{3};
    constexpr double EXPECTED_FP{73.7};
    constexpr DummyStruct EXPECTED_STRUCT{422};
    uint64_t ptr_source{0};
    uint64_t* expected_ptr{&ptr_source};

    SutInt sut_int;
    volatile SutInt sut_volatile;
    SutFP sut_fp;
    SutPtr sut_ptr;
    SutStruct sut_struct;

    sut_int = EXPECTED_INT;
    sut_volatile = EXPECTED_INT;
    sut_fp = EXPECTED_FP;
    sut_ptr = expected_ptr;
    sut_struct = EXPECTED_STRUCT;

    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_INT));
    EXPECT_THAT(sut_fp.load(), Eq(EXPECTED_FP));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_ptr));
    EXPECT_THAT(sut_struct.load(), Eq(EXPECTED_STRUCT));
}

TYPED_TEST(Atomic_test, ConversionOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "364732a4-5456-435c-80bf-34f54172632c");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    constexpr uint64_t EXPECTED_INT{113};
    constexpr double EXPECTED_FP{7.37};
    constexpr DummyStruct EXPECTED_STRUCT{242};
    uint64_t ptr_source{0};
    uint64_t* expected_ptr{&ptr_source};

    SutInt sut_int{EXPECTED_INT};
    volatile SutInt sut_volatile{EXPECTED_INT};
    SutFP sut_fp{EXPECTED_FP};
    SutPtr sut_ptr(expected_ptr);
    SutStruct sut_struct{EXPECTED_STRUCT};

    EXPECT_THAT(static_cast<uint64_t>(sut_int), Eq(EXPECTED_INT));
    EXPECT_THAT(static_cast<uint64_t>(sut_volatile), Eq(EXPECTED_INT));
    EXPECT_THAT(static_cast<double>(sut_fp), Eq(EXPECTED_FP));
    EXPECT_THAT(static_cast<uint64_t*>(sut_ptr), Eq(expected_ptr));
    EXPECT_THAT(static_cast<DummyStruct>(sut_struct), Eq(EXPECTED_STRUCT));
}

TYPED_TEST(Atomic_test, IsLockFreeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d5eb1b2c-515d-42fd-87f2-b8f51dd29d09");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    SutInt sut_int;
    volatile SutInt sut_volatile;
    SutFP sut_fp;
    SutPtr sut_ptr;
    SutStruct sut_struct;

    EXPECT_THAT(sut_int.is_lock_free(), Eq(true));
    EXPECT_THAT(sut_volatile.is_lock_free(), Eq(true));
    EXPECT_THAT(sut_fp.is_lock_free(), Eq(true));
    EXPECT_THAT(sut_ptr.is_lock_free(), Eq(true));
    EXPECT_THAT(sut_struct.is_lock_free(), Eq(true));
}

TYPED_TEST(Atomic_test, StoreAndLoadWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "10d44cbe-81df-4d89-a6db-6553a5b58d02");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    constexpr uint64_t EXPECTED_INT{31};
    constexpr double EXPECTED_FP{73.73};
    constexpr DummyStruct EXPECTED_STRUCT{4422};
    uint64_t ptr_source{0};
    uint64_t* expected_ptr{&ptr_source};

    SutInt sut_int;
    volatile SutInt sut_volatile;
    SutFP sut_fp;
    SutPtr sut_ptr;
    SutStruct sut_struct;

    sut_int.store(EXPECTED_INT, std::memory_order::memory_order_relaxed);
    sut_volatile.store(EXPECTED_INT, std::memory_order::memory_order_relaxed);
    sut_fp.store(EXPECTED_FP, std::memory_order::memory_order_relaxed);
    sut_ptr.store(expected_ptr, std::memory_order::memory_order_relaxed);
    sut_struct.store(EXPECTED_STRUCT, std::memory_order::memory_order_relaxed);

    EXPECT_THAT(sut_int.load(std::memory_order::memory_order_relaxed), Eq(EXPECTED_INT));
    EXPECT_THAT(sut_volatile.load(std::memory_order::memory_order_relaxed), Eq(EXPECTED_INT));
    EXPECT_THAT(sut_fp.load(std::memory_order::memory_order_relaxed), Eq(EXPECTED_FP));
    EXPECT_THAT(sut_ptr.load(std::memory_order::memory_order_relaxed), Eq(expected_ptr));
    EXPECT_THAT(sut_struct.load(std::memory_order::memory_order_relaxed), Eq(EXPECTED_STRUCT));
}

TYPED_TEST(Atomic_test, ExchangeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "a461a0ee-bda6-45e4-a7f0-f501d3c385b4");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    constexpr uint64_t EXPECTED_INITIAL_INT{31};
    constexpr double EXPECTED_INITIAL_FP{73.73};
    constexpr DummyStruct EXPECTED_INITIAL_STRUCT{4422};
    uint64_t ptr_initial_source{0};
    uint64_t* expected_initial_ptr{&ptr_initial_source};

    constexpr uint64_t EXPECTED_NEW_INT{31};
    constexpr double EXPECTED_NEW_FP{73.73};
    constexpr DummyStruct EXPECTED_NEW_STRUCT{4422};
    uint64_t ptr_new_source{0};
    uint64_t* expected_new_ptr{&ptr_new_source};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutFP sut_fp{EXPECTED_INITIAL_FP};
    SutPtr sut_ptr(expected_initial_ptr);
    SutStruct sut_struct{EXPECTED_INITIAL_STRUCT};

    EXPECT_THAT(sut_int.exchange(EXPECTED_NEW_INT, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.exchange(EXPECTED_NEW_INT, std::memory_order::memory_order_relaxed),
                Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_fp.exchange(EXPECTED_NEW_FP, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_FP));
    EXPECT_THAT(sut_ptr.exchange(expected_new_ptr, std::memory_order::memory_order_relaxed), Eq(expected_initial_ptr));
    EXPECT_THAT(sut_struct.exchange(EXPECTED_NEW_STRUCT, std::memory_order::memory_order_relaxed),
                Eq(EXPECTED_INITIAL_STRUCT));

    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_fp.load(), Eq(EXPECTED_NEW_FP));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
    EXPECT_THAT(sut_struct.load(), Eq(EXPECTED_NEW_STRUCT));
}

TYPED_TEST(Atomic_test, CompareExchangeWeakWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d563ea82-4284-4b1c-8791-9387f5c77fc1");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    constexpr uint64_t EXPECTED_INITIAL_INT{31};
    constexpr double EXPECTED_INITIAL_FP{73.73};
    constexpr DummyStruct EXPECTED_INITIAL_STRUCT{4422};
    uint64_t ptr_initial_source{0};
    uint64_t* expected_initial_ptr{&ptr_initial_source};

    constexpr uint64_t EXPECTED_NEW_INT{313};
    constexpr double EXPECTED_NEW_FP{73.37};
    constexpr DummyStruct EXPECTED_NEW_STRUCT{2244};
    uint64_t ptr_new_source{0};
    uint64_t* expected_new_ptr{&ptr_new_source};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutFP sut_fp{EXPECTED_INITIAL_FP};
    SutPtr sut_ptr(expected_initial_ptr);
    SutStruct sut_struct{EXPECTED_INITIAL_STRUCT};

    uint64_t expected_int{EXPECTED_INITIAL_INT};
    EXPECT_THAT(sut_int.compare_exchange_weak(expected_int, EXPECTED_NEW_INT, std::memory_order::memory_order_relaxed),
                Eq(true));
    EXPECT_THAT(expected_int, Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.compare_exchange_weak(expected_int,
                                              EXPECTED_INITIAL_INT,
                                              std::memory_order::memory_order_release,
                                              std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_int, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));

    uint64_t expected_volatile{EXPECTED_INITIAL_INT};
    EXPECT_THAT(sut_volatile.compare_exchange_weak(
                    expected_volatile, EXPECTED_NEW_INT, std::memory_order::memory_order_relaxed),
                Eq(true));
    EXPECT_THAT(expected_volatile, Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.compare_exchange_weak(expected_volatile,
                                                   EXPECTED_INITIAL_INT,
                                                   std::memory_order::memory_order_release,
                                                   std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_volatile, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));

    double expected_fp{EXPECTED_INITIAL_FP};
    EXPECT_THAT(sut_fp.compare_exchange_weak(expected_fp, EXPECTED_NEW_FP, std::memory_order::memory_order_relaxed),
                Eq(true));
    EXPECT_THAT(expected_fp, Eq(EXPECTED_INITIAL_FP));
    EXPECT_THAT(sut_fp.load(), Eq(EXPECTED_NEW_FP));
    EXPECT_THAT(sut_fp.compare_exchange_weak(expected_fp,
                                             EXPECTED_INITIAL_FP,
                                             std::memory_order::memory_order_release,
                                             std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_fp, Eq(EXPECTED_NEW_FP));
    EXPECT_THAT(sut_fp.load(), Eq(EXPECTED_NEW_FP));

    uint64_t* expected_ptr{expected_initial_ptr};
    EXPECT_THAT(sut_ptr.compare_exchange_weak(expected_ptr, expected_new_ptr, std::memory_order::memory_order_relaxed),
                Eq(true));
    EXPECT_THAT(expected_ptr, Eq(expected_initial_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.compare_exchange_weak(expected_ptr,
                                              expected_initial_ptr,
                                              std::memory_order::memory_order_release,
                                              std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_ptr, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));

    DummyStruct expected_struct{EXPECTED_INITIAL_STRUCT};
    EXPECT_THAT(
        sut_struct.compare_exchange_weak(expected_struct, EXPECTED_NEW_STRUCT, std::memory_order::memory_order_relaxed),
        Eq(true));
    EXPECT_THAT(expected_struct, Eq(EXPECTED_INITIAL_STRUCT));
    EXPECT_THAT(sut_struct.load(), Eq(EXPECTED_NEW_STRUCT));
    EXPECT_THAT(sut_struct.compare_exchange_weak(expected_struct,
                                                 EXPECTED_INITIAL_STRUCT,
                                                 std::memory_order::memory_order_release,
                                                 std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_struct, Eq(EXPECTED_NEW_STRUCT));
    EXPECT_THAT(sut_struct.load(), Eq(EXPECTED_NEW_STRUCT));
}

TYPED_TEST(Atomic_test, CompareExchangeStrongWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9f39293c-5d84-4d59-ab91-58fb738b5386");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutFP = typename TestFixture::template SutType<double>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;
    using SutStruct = typename TestFixture::template SutType<DummyStruct>;

    constexpr uint64_t EXPECTED_INITIAL_INT{131};
    constexpr double EXPECTED_INITIAL_FP{37.73};
    constexpr DummyStruct EXPECTED_INITIAL_STRUCT{4242};
    uint64_t ptr_initial_source{0};
    uint64_t* expected_initial_ptr{&ptr_initial_source};

    constexpr uint64_t EXPECTED_NEW_INT{313};
    constexpr double EXPECTED_NEW_FP{73.37};
    constexpr DummyStruct EXPECTED_NEW_STRUCT{2244};
    uint64_t ptr_new_source{0};
    uint64_t* expected_new_ptr{&ptr_new_source};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutFP sut_fp{EXPECTED_INITIAL_FP};
    SutPtr sut_ptr(expected_initial_ptr);
    SutStruct sut_struct{EXPECTED_INITIAL_STRUCT};

    uint64_t expected_int{EXPECTED_INITIAL_INT};
    EXPECT_THAT(
        sut_int.compare_exchange_strong(expected_int, EXPECTED_NEW_INT, std::memory_order::memory_order_relaxed),
        Eq(true));
    EXPECT_THAT(expected_int, Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.compare_exchange_strong(expected_int,
                                                EXPECTED_INITIAL_INT,
                                                std::memory_order::memory_order_release,
                                                std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_int, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));

    uint64_t expected_volatile{EXPECTED_INITIAL_INT};
    EXPECT_THAT(sut_volatile.compare_exchange_strong(
                    expected_volatile, EXPECTED_NEW_INT, std::memory_order::memory_order_relaxed),
                Eq(true));
    EXPECT_THAT(expected_volatile, Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.compare_exchange_strong(expected_volatile,
                                                     EXPECTED_INITIAL_INT,
                                                     std::memory_order::memory_order_release,
                                                     std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_volatile, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));

    double expected_fp{EXPECTED_INITIAL_FP};
    EXPECT_THAT(sut_fp.compare_exchange_strong(expected_fp, EXPECTED_NEW_FP, std::memory_order::memory_order_relaxed),
                Eq(true));
    EXPECT_THAT(expected_fp, Eq(EXPECTED_INITIAL_FP));
    EXPECT_THAT(sut_fp.load(), Eq(EXPECTED_NEW_FP));
    EXPECT_THAT(sut_fp.compare_exchange_strong(expected_fp,
                                               EXPECTED_INITIAL_FP,
                                               std::memory_order::memory_order_release,
                                               std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_fp, Eq(EXPECTED_NEW_FP));
    EXPECT_THAT(sut_fp.load(), Eq(EXPECTED_NEW_FP));

    uint64_t* expected_ptr{expected_initial_ptr};
    EXPECT_THAT(
        sut_ptr.compare_exchange_strong(expected_ptr, expected_new_ptr, std::memory_order::memory_order_relaxed),
        Eq(true));
    EXPECT_THAT(expected_ptr, Eq(expected_initial_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.compare_exchange_strong(expected_ptr,
                                                expected_initial_ptr,
                                                std::memory_order::memory_order_release,
                                                std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_ptr, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));

    DummyStruct expected_struct{EXPECTED_INITIAL_STRUCT};
    EXPECT_THAT(sut_struct.compare_exchange_strong(
                    expected_struct, EXPECTED_NEW_STRUCT, std::memory_order::memory_order_relaxed),
                Eq(true));
    EXPECT_THAT(expected_struct, Eq(EXPECTED_INITIAL_STRUCT));
    EXPECT_THAT(sut_struct.load(), Eq(EXPECTED_NEW_STRUCT));
    EXPECT_THAT(sut_struct.compare_exchange_strong(expected_struct,
                                                   EXPECTED_INITIAL_STRUCT,
                                                   std::memory_order::memory_order_release,
                                                   std::memory_order::memory_order_acquire),
                Eq(false));
    EXPECT_THAT(expected_struct, Eq(EXPECTED_NEW_STRUCT));
    EXPECT_THAT(sut_struct.load(), Eq(EXPECTED_NEW_STRUCT));
}

TYPED_TEST(Atomic_test, FetchAddWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9525afc0-35cc-45ae-84d6-11f154395740");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;

    constexpr uint64_t DELTA_INT{5};
    constexpr std::ptrdiff_t DELTA_PTR{DELTA_INT};

    constexpr uint64_t EXPECTED_INITIAL_INT{31};
    uint64_t array[200];
    uint64_t* expected_initial_ptr{&array[100]};

    constexpr uint64_t EXPECTED_NEW_INT{EXPECTED_INITIAL_INT + DELTA_INT};
    uint64_t* expected_new_ptr{expected_initial_ptr + DELTA_PTR};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutPtr sut_ptr(expected_initial_ptr);

    EXPECT_THAT(sut_int.fetch_add(DELTA_INT, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.fetch_add(DELTA_INT, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_ptr.fetch_add(DELTA_PTR, std::memory_order::memory_order_relaxed), Eq(expected_initial_ptr));

    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
}

TYPED_TEST(Atomic_test, FetchSubWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "2b13c4c3-95d8-43d2-a8e2-7e93b1f1e892");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;

    constexpr uint64_t DELTA_INT{3};
    constexpr std::ptrdiff_t DELTA_PTR{DELTA_INT};

    constexpr uint64_t EXPECTED_INITIAL_INT{31};
    uint64_t array[200];
    uint64_t* expected_initial_ptr{&array[100]};

    constexpr uint64_t EXPECTED_NEW_INT{EXPECTED_INITIAL_INT - DELTA_INT};
    uint64_t* expected_new_ptr{expected_initial_ptr - DELTA_PTR};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutPtr sut_ptr(expected_initial_ptr);

    EXPECT_THAT(sut_int.fetch_sub(DELTA_INT, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.fetch_sub(DELTA_INT, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_ptr.fetch_sub(DELTA_PTR, std::memory_order::memory_order_relaxed), Eq(expected_initial_ptr));

    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
}

TYPED_TEST(Atomic_test, AddAssignmentWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "90bdd039-31d3-47ed-9d10-7c38e1e837ce");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;

    constexpr uint64_t DELTA_INT{13};
    constexpr std::ptrdiff_t DELTA_PTR{DELTA_INT};

    constexpr uint64_t EXPECTED_INITIAL_INT{31};
    uint64_t array[200];
    uint64_t* expected_initial_ptr{&array[100]};

    constexpr uint64_t EXPECTED_NEW_INT{EXPECTED_INITIAL_INT + DELTA_INT};
    uint64_t* expected_new_ptr{expected_initial_ptr + DELTA_PTR};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutPtr sut_ptr(expected_initial_ptr);

    EXPECT_THAT(sut_int += DELTA_INT, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));

    EXPECT_THAT(sut_volatile += DELTA_INT, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));

    EXPECT_THAT(sut_ptr += DELTA_PTR, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
}

TYPED_TEST(Atomic_test, SubAssignmentWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "56737ae3-67e7-4daf-a316-f25c19af7fb9");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;

    constexpr uint64_t DELTA_INT{10};
    constexpr std::ptrdiff_t DELTA_PTR{DELTA_INT};

    constexpr uint64_t EXPECTED_INITIAL_INT{31};
    uint64_t array[200];
    uint64_t* expected_initial_ptr{&array[100]};

    constexpr uint64_t EXPECTED_NEW_INT{EXPECTED_INITIAL_INT - DELTA_INT};
    uint64_t* expected_new_ptr{expected_initial_ptr - DELTA_PTR};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutPtr sut_ptr(expected_initial_ptr);

    EXPECT_THAT(sut_int -= DELTA_INT, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));

    EXPECT_THAT(sut_volatile -= DELTA_INT, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));

    EXPECT_THAT(sut_ptr -= DELTA_PTR, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
}

TYPED_TEST(Atomic_test, IncrementWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "91512bb5-f57c-485d-9b54-484c947e47e6");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;

    constexpr uint64_t DELTA_INT{1};
    constexpr std::ptrdiff_t DELTA_PTR{DELTA_INT};

    constexpr uint64_t EXPECTED_INITIAL_INT{73};
    uint64_t array[200];
    uint64_t* expected_initial_ptr{&array[100]};

    constexpr uint64_t EXPECTED_NEW_INT{EXPECTED_INITIAL_INT + DELTA_INT};
    uint64_t* expected_new_ptr{expected_initial_ptr + DELTA_PTR};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutPtr sut_ptr(expected_initial_ptr);

    EXPECT_THAT(++sut_int, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int++, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT + DELTA_INT));

    EXPECT_THAT(++sut_volatile, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile++, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT + DELTA_INT));

    EXPECT_THAT(++sut_ptr, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr++, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr + DELTA_PTR));
}

TYPED_TEST(Atomic_test, DecrementWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f74a3e4e-109e-47da-8250-8172de6d7cad");

    using SutInt = typename TestFixture::template SutType<uint64_t>;
    using SutPtr = typename TestFixture::template SutType<uint64_t*>;

    constexpr uint64_t DELTA_INT{1};
    constexpr std::ptrdiff_t DELTA_PTR{DELTA_INT};

    constexpr uint64_t EXPECTED_INITIAL_INT{42};
    uint64_t array[200];
    uint64_t* expected_initial_ptr{&array[100]};

    constexpr uint64_t EXPECTED_NEW_INT{EXPECTED_INITIAL_INT - DELTA_INT};
    uint64_t* expected_new_ptr{expected_initial_ptr - DELTA_PTR};

    SutInt sut_int{EXPECTED_INITIAL_INT};
    volatile SutInt sut_volatile{EXPECTED_INITIAL_INT};
    SutPtr sut_ptr(expected_initial_ptr);

    EXPECT_THAT(--sut_int, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int--, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT - DELTA_INT));

    EXPECT_THAT(--sut_volatile, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile--, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT - DELTA_INT));

    EXPECT_THAT(--sut_ptr, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr--, Eq(expected_new_ptr));
    EXPECT_THAT(sut_ptr.load(), Eq(expected_new_ptr - DELTA_PTR));
}

TYPED_TEST(Atomic_test, BitwiseAndWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "3bd8b380-81d1-4a26-82cf-43ce548a44ba");

    using SutInt = typename TestFixture::template SutType<uint64_t>;

    constexpr uint64_t MASK{0b1010};
    constexpr uint64_t EXPECTED_INITIAL_INT{0b1001};
    constexpr uint64_t EXPECTED_NEW_INT{0b1000};

    SutInt sut_int;
    volatile SutInt sut_volatile;

    sut_int = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_int.fetch_and(MASK, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    sut_int = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_int &= MASK, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));

    sut_volatile = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_volatile.fetch_and(MASK, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    sut_volatile = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_volatile &= MASK, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
}

TYPED_TEST(Atomic_test, BitwiseOrWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f442c26c-2f22-43df-90a0-8a9a4fff03b3");

    using SutInt = typename TestFixture::template SutType<uint64_t>;

    constexpr uint64_t MASK{0b1010};
    constexpr uint64_t EXPECTED_INITIAL_INT{0b1001};
    constexpr uint64_t EXPECTED_NEW_INT{0b1011};

    SutInt sut_int;
    volatile SutInt sut_volatile;

    sut_int = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_int.fetch_or(MASK, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    sut_int = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_int |= MASK, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));

    sut_volatile = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_volatile.fetch_or(MASK, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    sut_volatile = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_volatile |= MASK, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
}

TYPED_TEST(Atomic_test, BitwiseXorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b7389a74-0eb4-4898-9d6d-bbf0a2d0aea7");

    using SutInt = typename TestFixture::template SutType<uint64_t>;

    constexpr uint64_t MASK{0b1010};
    constexpr uint64_t EXPECTED_INITIAL_INT{0b1001};
    constexpr uint64_t EXPECTED_NEW_INT{0b0011};

    SutInt sut_int;
    volatile SutInt sut_volatile;

    sut_int = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_int.fetch_xor(MASK, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));
    sut_int = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_int ^= MASK, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_int.load(), Eq(EXPECTED_NEW_INT));

    sut_volatile = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_volatile.fetch_xor(MASK, std::memory_order::memory_order_relaxed), Eq(EXPECTED_INITIAL_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
    sut_volatile = EXPECTED_INITIAL_INT;
    EXPECT_THAT(sut_volatile ^= MASK, Eq(EXPECTED_NEW_INT));
    EXPECT_THAT(sut_volatile.load(), Eq(EXPECTED_NEW_INT));
}

} // namespace
