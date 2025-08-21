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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_TESTING_TEST_UTILS_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_TESTING_TEST_UTILS_HPP

#include <type_traits>

namespace iox2 {
namespace container {
namespace testing {

// An opaque function call that prevents the compiler from making arbitrary assumptions about how an object is used.
void opaque_use(void*);
void opaque_use(void const*);

template<typename T, std::enable_if_t<!std::is_pointer<T>::value, bool> = true>
void opaque_use(T& object) {
    opaque_use(&object);
}

template<typename T, std::enable_if_t<!std::is_pointer<T>::value, bool> = true>
void opaque_use(T const& object) {
    opaque_use(&object);
}

} // namespace testing
} // namespace container
} // namespace iox2

#endif
