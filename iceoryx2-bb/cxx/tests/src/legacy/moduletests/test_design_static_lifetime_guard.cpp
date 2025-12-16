// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox2/legacy/static_lifetime_guard.hpp"

#include "iox2/legacy/testing/barrier.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <chrono>
#include <thread>
#include <vector>

namespace {
using namespace ::testing;
using namespace ::iox2::legacy::testing;

// allows to create different types for independent tests
template <uint64_t N>
struct Fou {
    Fou()
        : id(++instancesCreated) {
        ++ctorCalled;
    }

    ~Fou() {
        ++dtorCalled;
    }

    static void reset() {
        ctorCalled = 0;
        dtorCalled = 0;
    }

    static uint32_t ctorCalled;
    static uint32_t dtorCalled;
    static uint32_t instancesCreated;

    uint32_t id;
};

// delay is only needed for multithreaded test
template <uint64_t N>
struct DelayedFou : public Fou<N> {
    explicit DelayedFou(std::chrono::nanoseconds delay) {
        std::this_thread::sleep_for(delay);
    }
};

constexpr uint32_t FIRST_INSTANCE_ID { 1 };
constexpr uint32_t SECOND_INSTANCE_ID { 2 };

template <uint64_t N>
uint32_t Fou<N>::ctorCalled { 0 };

template <uint64_t N>
uint32_t Fou<N>::dtorCalled { 0 };

template <uint64_t N>
uint32_t Fou<N>::instancesCreated { 0 };

template <uint64_t N>
using TestGuard = iox2::legacy::StaticLifetimeGuard<Fou<N>>;

// create a bundle of types and functions that are relevant for the tests,
// since we need a different static type for each test
// to ensure independence
template <uint64_t N>
struct TestTypes : public TestGuard<N> {
    // NB: using the base class methods would admit to argue that we test another type,
    // hence we use this alias of the Guard
    using Guard = TestGuard<N>;
    using Foo = Fou<N>;

    using TestGuard<N>::setCount;

    // the first call to testInstance() creates a static instance
    // that is guarded once implicitly

    static Foo& instance() {
        static Foo& f = Guard::instance();
        return f;
    }

    // init the instance but also reset the Foo ctor count,
    // used at start of some tests to simplify counting
    static Foo& initInstance() {
        static Foo& f = Guard::instance();
        Foo::reset();
        return f;
    }
};

// each test must use a different N, __LINE__ is unique and portable,
// __COUNTER__ is not portable
// shorten the initialization via macro, needed due to __LINE__
#define UNIQUE_TYPE TestTypes<__LINE__>

// each test uses its own type for maximum separation, so we do not need to reset anything
class StaticLifetimeGuard_test : public Test {
  public:
    void SetUp() override {
    }

    void TearDown() override {
    }
};

TEST_F(StaticLifetimeGuard_test, countIsZeroIfNoInstanceExists) {
    ::testing::Test::RecordProperty("TEST_ID", "0bf772c8-97c7-4cdb-80a1-e1b6a1a4fdc6");
    using T = UNIQUE_TYPE;

    EXPECT_EQ(T::Guard::count(), 0);
    EXPECT_EQ(T::Foo::ctorCalled, 0);
    EXPECT_EQ(T::Foo::dtorCalled, 0);
}

TEST_F(StaticLifetimeGuard_test, guardDoesNotImplyInstanceConstructionIfInstanceIsNotCreated) {
    ::testing::Test::RecordProperty("TEST_ID", "0db1455e-1b1f-4498-af3c-5e2d7e92180b");
    using T = UNIQUE_TYPE;

    {
        T::Guard g;
        EXPECT_EQ(T::Guard::count(), 1);
    }

    EXPECT_EQ(T::Guard::count(), 0);
    EXPECT_EQ(T::Foo::ctorCalled, 0);
    EXPECT_EQ(T::Foo::dtorCalled, 0);
}

TEST_F(StaticLifetimeGuard_test, staticInitializationSucceeded) {
    ::testing::Test::RecordProperty("TEST_ID", "d38b436b-f079-43fe-9d33-23d18cd08ffc");
    using T = UNIQUE_TYPE;

    // testInstance() was constructed and the instance still exists
    EXPECT_EQ(T::instance().id, FIRST_INSTANCE_ID);
    EXPECT_EQ(T::Guard::count(), 1);
    EXPECT_EQ(T::Foo::ctorCalled, 1);
    EXPECT_EQ(T::Foo::dtorCalled, 0);
}

// setCount is not part of the public interface but still useful to check whether it works
TEST_F(StaticLifetimeGuard_test, setCountWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "1db790f9-d49e-44b2-b7e9-af50dd6a7d67");
    using T = UNIQUE_TYPE;

    T::Guard guard;
    auto oldCount = T::setCount(73);
    EXPECT_EQ(T::Guard::count(), 73);
    EXPECT_EQ(oldCount, 1);
}

TEST_F(StaticLifetimeGuard_test, guardPreventsDestruction) {
    ::testing::Test::RecordProperty("TEST_ID", "5a8c5953-f2d7-4539-89ba-b4686bbb6319");
    using T = UNIQUE_TYPE;
    T::initInstance();

    EXPECT_EQ(T::instance().id, FIRST_INSTANCE_ID);
    {
        T::Guard guard;
        EXPECT_EQ(T::count(), 2);
        auto& instance = T::Guard::instance();

        EXPECT_EQ(T::Foo::ctorCalled, 0);
        EXPECT_EQ(T::Foo::dtorCalled, 0);

        // still the same instance as testInstance()
        EXPECT_EQ(instance.id, FIRST_INSTANCE_ID);
        EXPECT_EQ(&instance, &T::instance());
    }

    // the implicit guard of testInstance() prevents destruction
    EXPECT_EQ(T::Foo::ctorCalled, 0);
    EXPECT_EQ(T::Foo::dtorCalled, 0);
    EXPECT_EQ(T::instance().id, FIRST_INSTANCE_ID);
}

TEST_F(StaticLifetimeGuard_test, copyIncreasesLifetimeCount) {
    ::testing::Test::RecordProperty("TEST_ID", "6ab6396d-7c63-4626-92ed-c7f3ea67bbf1");
    using T = UNIQUE_TYPE;
    T::initInstance();

    EXPECT_EQ(T::instance().id, FIRST_INSTANCE_ID);

    T::Guard guard;
    {
        EXPECT_EQ(T::Guard::count(), 2);
        // NOLINTJUSTIFICATION ctor and dtor side effects are tested
        // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
        T::Guard copy(guard);
        EXPECT_EQ(T::Guard::count(), 3);
    }
    EXPECT_EQ(T::Guard::count(), 2);

    EXPECT_EQ(T::Foo::ctorCalled, 0);
    EXPECT_EQ(T::Foo::dtorCalled, 0);
}

TEST_F(StaticLifetimeGuard_test, moveIncreasesLifetimeCount) {
    ::testing::Test::RecordProperty("TEST_ID", "32a2fdbf-cb02-408c-99a3-373aa66b2764");
    using T = UNIQUE_TYPE;
    T::initInstance();

    T::Guard guard;
    {
        EXPECT_EQ(T::Guard::count(), 2);
        // NOLINTJUSTIFICATION ctor and dtor side effects are tested
        // NOLINTNEXTLINE(performance-unnecessary-copy-initialization)
        T::Guard movedGuard(std::move(guard));
        EXPECT_EQ(T::Guard::count(), 3);
    }
    EXPECT_EQ(T::Guard::count(), 2);

    EXPECT_EQ(T::Foo::ctorCalled, 0);
    EXPECT_EQ(T::Foo::dtorCalled, 0);
}

TEST_F(StaticLifetimeGuard_test, assignmentDoesNotChangeLifetimeCount) {
    ::testing::Test::RecordProperty("TEST_ID", "1c04ac75-d47a-44da-b8dc-6f567a53d3fc");
    using T = UNIQUE_TYPE;
    T::initInstance();

    T::Guard guard1;
    T::Guard guard2;

    EXPECT_EQ(T::Guard::count(), 3);
    guard1 = guard2;
    EXPECT_EQ(T::Guard::count(), 3);
    guard1 = std::move(guard2);
    EXPECT_EQ(T::Guard::count(), 3);

    EXPECT_EQ(T::Foo::ctorCalled, 0);
    EXPECT_EQ(T::Foo::dtorCalled, 0);
}

TEST_F(StaticLifetimeGuard_test, destructionAtZeroCountWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "8b5a22a9-87bc-434b-9d07-9f3c20a6944e");
    using T = UNIQUE_TYPE;
    T::initInstance();

    {
        T::Guard guard;
        auto& instance = T::Guard::instance();

        // count is expected to be 2,
        // we ignore the guard of testInstance() by setting it to 1,
        // hence when guard is destroyed the instance will be destroyed as well
        auto oldCount = T::setCount(1);
        ASSERT_EQ(oldCount, 2);

        EXPECT_EQ(T::Foo::ctorCalled, 0);
        EXPECT_EQ(T::Foo::dtorCalled, 0);
        EXPECT_EQ(instance.id, FIRST_INSTANCE_ID);
    }

    EXPECT_EQ(T::Guard::count(), 0);
    EXPECT_EQ(T::Foo::ctorCalled, 0);
    EXPECT_EQ(T::Foo::dtorCalled, 1);
}

TEST_F(StaticLifetimeGuard_test, constructionAfterDestructionWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "0077e73d-ddf5-47e7-a7c6-93819f376175");
    using T = UNIQUE_TYPE;
    T::initInstance();

    {
        T::Guard guard;
        auto& instance = T::Guard::instance();

        T::setCount(1);
        EXPECT_EQ(instance.id, FIRST_INSTANCE_ID);
    }

    // first instance destroyed (should usually only happen at the end of the program
    // during static destruction)

    T::Foo::reset();

    EXPECT_EQ(T::Guard::count(), 0);
    {
        T::Guard guard;
        auto& instance = T::Guard::instance();

        EXPECT_EQ(T::Foo::ctorCalled, 1);
        EXPECT_EQ(T::Foo::dtorCalled, 0);
        EXPECT_EQ(instance.id, SECOND_INSTANCE_ID);
    }

    // there was only one guard for the second instance that is destroyed
    // at scope end and hence the second instance should be destroyed as well

    EXPECT_EQ(T::Guard::count(), 0);
    EXPECT_EQ(T::Foo::ctorCalled, 1);
    EXPECT_EQ(T::Foo::dtorCalled, 1);
}

// note that this test cannot guarantee concurrent correctness due to thread scheduling
// being unpredictable
TEST_F(StaticLifetimeGuard_test, instanceCtorIsConcurrentlyCalledExactlyOnce) {
    ::testing::Test::RecordProperty("TEST_ID", "2b7e60e5-159d-4bcf-adc8-21f5a23d2f27");
    using Instance = DelayedFou<1>;
    using Sut = iox2::legacy::StaticLifetimeGuard<Instance>;
    constexpr uint32_t NUM_THREADS = 8;

    EXPECT_EQ(Instance::ctorCalled, 0);

    // wait at the barrier to ensure threads were started and increase the
    // concurrent execution probability (but cannot guarantee concurrent execution)
    Barrier barrier(NUM_THREADS);
    auto createInstance = [&barrier]() {
        barrier.notify();
        barrier.wait();
        // all threads have notfied (but may pass wait in any order ...)

        // cannot wait too long otherwise we slow down the tests too much,
        // cannot be optimized away, as it has side effects (counting)
        Sut::instance(std::chrono::milliseconds(1));
    };

    std::vector<std::thread> threads;
    threads.reserve(NUM_THREADS);

    for (uint32_t i = 0; i < NUM_THREADS; ++i) {
        threads.emplace_back(createInstance);
    }

    // each join can only return once each thread arrived at the barrier and
    // called Sut::instance()
    for (auto& t : threads) {
        t.join();
    }

    EXPECT_EQ(Instance::ctorCalled, 1);
}

} // namespace
