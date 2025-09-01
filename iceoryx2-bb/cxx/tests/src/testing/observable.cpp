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

Observable::Observable() {
    ++s_counter.was_initialized;
    ++s_counter.total_instances;
}

Observable::Observable(int32_t object_id)
    : Observable() {
    id = object_id;
}

Observable::~Observable() {
    ++s_counter.was_destructed;
    --s_counter.total_instances;
}

Observable::Observable(Observable const& rhs)
    : id(rhs.id) {
    ++s_counter.was_copy_constructed;
    ++s_counter.total_instances;
}

Observable::Observable(Observable&& rhs) noexcept
    : id(std::exchange(rhs.id, 0)) {
    ++s_counter.was_move_constructed;
    ++s_counter.total_instances;
    rhs.id = 0;
}

auto Observable::operator=(Observable const& rhs) -> Observable& {
    if (this != &rhs) {
        id = rhs.id;
    }
    ++s_counter.was_copy_assigned;
    return *this;
}

auto Observable::operator=(Observable&& rhs) noexcept -> Observable& {
    id = std::exchange(rhs.id, 0);
    ++s_counter.was_move_assigned;
    return *this;
}

DetectLeakedObservablesFixture::DetectLeakedObservablesFixture()
    : m_is_armed(false) {
}

DetectLeakedObservablesFixture::~DetectLeakedObservablesFixture() = default;

void DetectLeakedObservablesFixture::SetUp() {
    Observable::reset_all_counters();
    ASSERT_EQ(Observable::s_counter.total_instances, 0);
    m_is_armed = true;
}

void DetectLeakedObservablesFixture::TearDown() {
    if (m_is_armed) {
        ASSERT_EQ(Observable::s_counter.total_instances, 0) << "Some Observable objects were not destructed properly";
    }
}

auto DetectLeakedObservablesFixture::has_leaked_observables() const -> bool {
    return Observable::s_counter.total_instances != 0;
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
    ASSERT_EQ(Observable::s_counter.was_initialized, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, 0);
    ASSERT_EQ(Observable::s_counter.was_move_constructed, 0);
    ASSERT_EQ(Observable::s_counter.was_move_assigned, 0);
    ASSERT_EQ(Observable::s_counter.was_destructed, 0);
    ASSERT_EQ(Observable::s_counter.total_instances, 0);
    m_expected = Observable::Counters {};
}

//NOLINTNEXTLINE(readability-function-cognitive-complexity,readability-function-size), triggered by gtest
void VerifyAllObservableInteractionsFixture::TearDown() {
    ASSERT_EQ(Observable::s_counter.was_initialized, m_expected.was_initialized);
    ASSERT_EQ(Observable::s_counter.was_copy_constructed, m_expected.was_copy_constructed);
    ASSERT_EQ(Observable::s_counter.was_copy_assigned, m_expected.was_copy_assigned);
    ASSERT_EQ(Observable::s_counter.was_move_constructed, m_expected.was_move_constructed);
    ASSERT_EQ(Observable::s_counter.was_move_assigned, m_expected.was_move_assigned);
    ASSERT_EQ(Observable::s_counter.was_destructed, m_expected.was_destructed);
    ASSERT_EQ(Observable::s_counter.total_instances, m_expected.total_instances);
}

auto VerifyAllObservableInteractionsFixture::expected_count() -> Observable::Counters& {
    return m_expected;
}

} // namespace testing
} // namespace container
} // namespace iox2
