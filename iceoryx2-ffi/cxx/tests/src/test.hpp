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

#ifndef IOX2_CXX_TESTS_TEST_HPP
#define IOX2_CXX_TESTS_TEST_HPP

#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <chrono>

using namespace ::testing;

namespace iox2_testing {
using namespace iox2;
template <ServiceType T>
struct TypeServiceType {
    static constexpr ServiceType TYPE = T;
};
using ServiceTypeIpc = TypeServiceType<ServiceType::Ipc>;
using ServiceTypeLocal = TypeServiceType<ServiceType::Local>;

using ServiceTypes = ::testing::Types<ServiceTypeIpc, ServiceTypeLocal>;

inline auto generate_service_name() -> ServiceName {
    static std::atomic<uint64_t> COUNTER = 0;
    const auto now = std::chrono::system_clock::now().time_since_epoch().count();
    const auto random_number = rand(); // NOLINT(cert-msc30-c,cert-msc50-cpp)
    return ServiceName::create((std::string("test_") + std::to_string(COUNTER.fetch_add(1)) + "_" + std::to_string(now)
                                + "_" + std::to_string(random_number))
                                   .c_str())
        .expect("");
}
} // namespace iox2_testing

#endif // IOX2_CXX_TESTS_TEST_HPP
