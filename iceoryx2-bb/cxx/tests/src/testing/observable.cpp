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

void Observable::resetAllCounters() {
    s_counter = Observable::Counters {};
}

Observable::Observable()
    : id(0) {
    ++s_counter.wasInitialized;
    ++s_counter.totalInstances;
}

Observable::Observable(int id)
    : Observable() {
    this->id = id;
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

Observable& Observable::operator=(Observable const& rhs) {
    id = rhs.id;
    ++s_counter.wasCopyAssigned;
    return *this;
}

Observable& Observable::operator=(Observable&& rhs) noexcept {
    id = rhs.id;
    ++s_counter.wasMoveAssigned;
    return *this;
}

DetectLeakedObservablesFixture::DetectLeakedObservablesFixture()
    : m_isArmed(false) {
}

DetectLeakedObservablesFixture::~DetectLeakedObservablesFixture() = default;

void DetectLeakedObservablesFixture::SetUp() {
    Observable::resetAllCounters();
    m_isArmed = true;
}

void DetectLeakedObservablesFixture::TearDown() {
    if (m_isArmed) {
        ASSERT_EQ(Observable::s_counter.totalInstances, 0) << "Some Observable objects were not destructed properly";
    }
}

bool DetectLeakedObservablesFixture::hasLeakedObservables() const {
    return Observable::s_counter.totalInstances != 0;
}

void DetectLeakedObservablesFixture::defuseLeakCheck() {
    m_isArmed = false;
}

} // namespace testing
} // namespace container
} // namespace iox2
