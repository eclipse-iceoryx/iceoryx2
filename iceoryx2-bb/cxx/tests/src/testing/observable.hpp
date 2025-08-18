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

#ifndef INCLUDE_GUARD_IOX2_CONTAINER_TESTING_OBSERVABLE_HPP
#define INCLUDE_GUARD_IOX2_CONTAINER_TESTING_OBSERVABLE_HPP

#include <gtest/gtest.h>

namespace iox2 {
namespace container {
namespace testing {

/// A mock type that tracks invocations of all special member functions and maintains an id for tracking copies.
class Observable {
  public:
    /// Counters keeping track of all operations performed on objects of type Observable.
    static struct Counters {
        int wasInitialized;     ///< Incremented for each invocation of a constructor other than copy or move.
        int wasCopyConstructed; ///< Incremented for each invocation of the copy constructor.
        int wasCopyAssigned;    ///< Incremented for each invocation of the copy assignment operator.
        int wasMoveConstructed; ///< Incremented for each invocation of the move constructor.
        int wasMoveAssigned;    ///< Incremented for each invocation of the move assignment operator.
        int wasDestructed;      ///< Incremented for each invocation of the destructor.
        int totalInstances;     ///< Incremented for each constructor, decremented for each destructor invocation.
    } s_counter;                ///< Static counters for Observable.
  public:
    int id; ///< Id of this object. Ids propagate on copy/move construction and assignment.
  public:
    /// Sets all counters in s_counter to 0.
    static void resetAllCounters();

    Observable();
    explicit Observable(int id);
    ~Observable();
    Observable(Observable const& rhs);
    Observable(Observable&& rhs) noexcept;
    Observable& operator=(Observable const& rhs);
    Observable& operator=(Observable&& rhs) noexcept;
};

/// A fixture that asserts that no instances of Observable were leaked after the completion of a test.
class DetectLeakedObservablesFixture : public ::testing::Test {
  private:
    bool m_isArmed;

  public:
    DetectLeakedObservablesFixture();
    ~DetectLeakedObservablesFixture() override;

    void SetUp() override;
    void TearDown() override;

    /// Checks whether there are currently any active instances of Observable that await destruction.
    bool hasLeakedObservables() const;
    /// Do not perform the check for leaks after this test.
    void defuseLeakCheck();
};

} // namespace testing
} // namespace container
} // namespace iox2

#endif
