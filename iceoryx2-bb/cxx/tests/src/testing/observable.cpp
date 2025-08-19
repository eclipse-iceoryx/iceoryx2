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
    : id(rhs.id) {
    ++s_counter.wasMoveConstructed;
    ++s_counter.totalInstances;
}

auto Observable::operator=(Observable const& rhs) -> Observable& {
    if (this != &rhs) {
        id = rhs.id;
    }
    ++s_counter.wasCopyAssigned;
    return *this;
}

auto Observable::operator=(Observable&& rhs) noexcept -> Observable& {
    id = rhs.id;
    ++s_counter.wasMoveAssigned;
    return *this;
}

DetectLeakedObservablesFixture::DetectLeakedObservablesFixture()
    : m_is_armed(false) {
}

DetectLeakedObservablesFixture::~DetectLeakedObservablesFixture() = default;

void DetectLeakedObservablesFixture::SetUp() {
    Observable::reset_all_counters();
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

} // namespace testing
} // namespace container
} // namespace iox2
