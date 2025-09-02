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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_TESTING_TEST_UTILS_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_TESTING_TEST_UTILS_HPP

#include <cstdint>
#include <type_traits>

namespace iox2 {
namespace container {
namespace testing {

// An opaque function call that prevents the compiler from making arbitrary assumptions about how an object is used.
void opaque_use(void* object);
void opaque_use(void const* object);

// NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
template <typename T, std::enable_if_t<!std::is_pointer<T>::value, bool> = true>
void opaque_use(T& object) {
    opaque_use(&object);
}

// NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
template <typename T, std::enable_if_t<!std::is_pointer<T>::value, bool> = true>
void opaque_use(T const& object) {
    opaque_use(&object);
}

// A class that overloads operator&, the address-of operator.
// The operator behaves the same as the built-in operator& but increments
// the static counter CustomAddressOperator::s_count_address_operator as
// a side-effect to make it detectable during testing.
class CustomAddressOperator {
  public:
    static int32_t s_count_address_operator;

    // NOLINTNEXTLINE(misc-non-private-member-variables-in-classes), exposed for testability
    int32_t id = 0;

    auto operator&() -> CustomAddressOperator* {
        ++s_count_address_operator;
        return this;
    }

    auto operator&() const -> CustomAddressOperator const* {
        ++s_count_address_operator;
        return this;
    }
};

} // namespace testing
} // namespace container
} // namespace iox2

#endif
