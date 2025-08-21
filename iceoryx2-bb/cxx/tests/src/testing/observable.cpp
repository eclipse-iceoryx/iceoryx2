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

#include "testing/observable.hpp"

#include <gtest/gtest.h>

#include <utility>

namespace iox2 {
namespace container {
namespace testing {

Observable::Counters Observable::s_counter = {};

void Observable::reset_all_counters() {
    s_counter = Observable::Counters {};
}

Observable::Observable()
    : id(0) {
    ++s_counter.wasInitialized;
    ++s_counter.totalInstances;
}

Observable::Observable(int object_id)
    : Observable() {
    id = object_id;
}

Observable::~Observable() {
    ++s_counter.wasDestructed;
    --s_counter.totalInstances;
}

Observable::Observable(Observable const& rhs)
    : id(rhs.id) {
    ++s_counter.wasCopyConstructed;
    ++s_counter.totalInstances;
}

Observable::Observable(Observable&& rhs) noexcept
    : id(std::exchange(rhs.id, 0)) {
    ++s_counter.wasMoveConstructed;
    ++s_counter.totalInstances;
    rhs.id = 0;
}

auto Observable::operator=(Observable const& rhs) -> Observable& {
    if (this != &rhs) {
        id = rhs.id;
    }
    ++s_counter.wasCopyAssigned;
    return *this;
}

auto Observable::operator=(Observable&& rhs) noexcept -> Observable& {
    id = std::exchange(rhs.id, 0);
    ++s_counter.wasMoveAssigned;
    return *this;
}

DetectLeakedObservablesFixture::DetectLeakedObservablesFixture()
    : m_is_armed(false) {
}

DetectLeakedObservablesFixture::~DetectLeakedObservablesFixture() = default;

void DetectLeakedObservablesFixture::SetUp() {
    Observable::reset_all_counters();
    ASSERT_EQ(Observable::s_counter.totalInstances, 0);
    m_is_armed = true;
}

void DetectLeakedObservablesFixture::TearDown() {
    if (m_is_armed) {
        ASSERT_EQ(Observable::s_counter.totalInstances, 0) << "Some Observable objects were not destructed properly";
    }
}

auto DetectLeakedObservablesFixture::has_leaked_observables() const -> bool {
    return Observable::s_counter.totalInstances != 0;
}

void DetectLeakedObservablesFixture::defuse_leak_check() {
    m_is_armed = false;
}

VerifyAllObservableInteractionsFixture::VerifyAllObservableInteractionsFixture()
    : m_expected {} {
}

VerifyAllObservableInteractionsFixture::~VerifyAllObservableInteractionsFixture() = default;

//NOLINTNEXTLINE(readability-function-cognitive-complexity,readability-function-size), triggered by gtest
void VerifyAllObservableInteractionsFixture::SetUp() {
    ASSERT_EQ(Observable::s_counter.wasInitialized, 0);
    ASSERT_EQ(Observable::s_counter.wasCopyConstructed, 0);
    ASSERT_EQ(Observable::s_counter.wasCopyAssigned, 0);
    ASSERT_EQ(Observable::s_counter.wasMoveConstructed, 0);
    ASSERT_EQ(Observable::s_counter.wasMoveAssigned, 0);
    ASSERT_EQ(Observable::s_counter.wasDestructed, 0);
    ASSERT_EQ(Observable::s_counter.totalInstances, 0);
    m_expected = Observable::Counters {};
}

//NOLINTNEXTLINE(readability-function-cognitive-complexity,readability-function-size), triggered by gtest
void VerifyAllObservableInteractionsFixture::TearDown() {
    ASSERT_EQ(Observable::s_counter.wasInitialized, m_expected.wasInitialized);
    ASSERT_EQ(Observable::s_counter.wasCopyConstructed, m_expected.wasCopyConstructed);
    ASSERT_EQ(Observable::s_counter.wasCopyAssigned, m_expected.wasCopyAssigned);
    ASSERT_EQ(Observable::s_counter.wasMoveConstructed, m_expected.wasMoveConstructed);
    ASSERT_EQ(Observable::s_counter.wasMoveAssigned, m_expected.wasMoveAssigned);
    ASSERT_EQ(Observable::s_counter.wasDestructed, m_expected.wasDestructed);
    ASSERT_EQ(Observable::s_counter.totalInstances, m_expected.totalInstances);
}

auto VerifyAllObservableInteractionsFixture::expected_count() -> Observable::Counters& {
    return m_expected;
}

} // namespace testing
} // namespace container
} // namespace iox2
