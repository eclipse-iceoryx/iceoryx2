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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_TESTING_OBSERVABLE_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_TESTING_OBSERVABLE_HPP

#include <gtest/gtest.h>

#include <cstdint>

namespace iox2 {
namespace container {
namespace testing {

/// A mock type that tracks invocations of all special member functions and maintains an id for tracking copies.
class Observable {
  public:
    /// Counters keeping track of all operations performed on objects of type Observable.
    static struct Counters {
        int32_t was_initialized;      ///< Incremented for each invocation of a constructor other than copy or move.
        int32_t was_copy_constructed; ///< Incremented for each invocation of the copy constructor.
        int32_t was_copy_assigned;    ///< Incremented for each invocation of the copy assignment operator.
        int32_t was_move_constructed; ///< Incremented for each invocation of the move constructor.
        int32_t was_move_assigned;    ///< Incremented for each invocation of the move assignment operator.
        int32_t was_destructed;       ///< Incremented for each invocation of the destructor.
        int32_t total_instances;      ///< Incremented for each constructor, decremented for each destructor invocation.
    } s_counter;                      ///< Static counters for Observable.

    // NOLINTNEXTLINE(misc-non-private-member-variables-in-classes), exposed for testability
    int32_t id = 0; ///< Id of this object. Ids propagate on copy/move construction and assignment.

    /// Sets all counters in s_counter to 0.
    static void reset_all_counters();

    Observable();
    explicit Observable(int32_t object_id);
    ~Observable();
    Observable(Observable const& rhs);
    Observable(Observable&& rhs) noexcept;
    auto operator=(Observable const& rhs) -> Observable&;
    auto operator=(Observable&& rhs) noexcept -> Observable&;
};

/// A fixture that asserts that no instances of Observable were leaked after the completion of a test.
class DetectLeakedObservablesFixture : public ::testing::Test {
  private:
    bool m_is_armed = true;

  public:
    DetectLeakedObservablesFixture();
    ~DetectLeakedObservablesFixture() override;

    DetectLeakedObservablesFixture(DetectLeakedObservablesFixture const&) = delete;
    DetectLeakedObservablesFixture(DetectLeakedObservablesFixture&&) = delete;
    auto operator=(DetectLeakedObservablesFixture const&) -> DetectLeakedObservablesFixture& = delete;
    auto operator=(DetectLeakedObservablesFixture&&) -> DetectLeakedObservablesFixture& = delete;

    void SetUp() override;
    void TearDown() override;

    /// Checks whether there are currently any active instances of Observable that await destruction.
    auto has_leaked_observables() const -> bool;
    /// Do not perform the check for leaks after this test.
    void defuse_leak_check();
};

// A fixture that checks all Observable counters against a set of expected values after the completion of a test.
class VerifyAllObservableInteractionsFixture : public ::testing::Test {
  private:
    Observable::Counters m_expected;

  public:
    VerifyAllObservableInteractionsFixture();
    ~VerifyAllObservableInteractionsFixture() override;

    VerifyAllObservableInteractionsFixture(VerifyAllObservableInteractionsFixture const&) = delete;
    VerifyAllObservableInteractionsFixture(VerifyAllObservableInteractionsFixture&&) = delete;
    auto operator=(VerifyAllObservableInteractionsFixture const&) -> VerifyAllObservableInteractionsFixture& = delete;
    auto operator=(VerifyAllObservableInteractionsFixture&&) -> VerifyAllObservableInteractionsFixture& = delete;

    void SetUp() override;
    void TearDown() override;

    // Retrieves the set of expected counter values that will be used for the check after this test.
    auto expected_count() -> Observable::Counters&;
};

} // namespace testing
} // namespace container
} // namespace iox2

#endif
