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

#include "iox2/component-tests/common.hpp"

#include <iox2/bb/static_string.hpp>
#include <iox2/iceoryx2.hpp>

#include <algorithm>
#include <iostream>
#include <vector>

namespace {
constexpr const uint64_t TEST_NAME_LENGTH = 32;

struct Test {
    // NOLINTBEGIN(misc-non-private-member-variables-in-classes)
    iox2::bb::StaticString<TEST_NAME_LENGTH> test_name;
    std::unique_ptr<IComponentTest> test;
    // NOLINTEND(misc-non-private-member-variables-in-classes)

    Test(char const* test_name_c_str, std::unique_ptr<IComponentTest> test)
        : test_name(*iox2::bb::StaticString<TEST_NAME_LENGTH>::from_utf8_null_terminated_unchecked(test_name_c_str))
        , test(std::move(test)) {
    }
};

void add_test(std::vector<Test>& tests_vector, std::unique_ptr<IComponentTest> ptest);

auto component_tests() -> std::vector<Test> {
    std::vector<Test> ret;
    add_test(ret, test_containers());
    add_test(ret, test_container_mutation());
    return ret;
}

void add_test(std::vector<Test>& tests_vector, std::unique_ptr<IComponentTest> ptest) {
    char const* test_name = ptest->test_name();
    tests_vector.emplace_back(test_name, std::move(ptest));
}
} // namespace

struct ComponentTestHeader {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "ComponentTestHeader";

    iox2::bb::StaticString<TEST_NAME_LENGTH> test_name;
};


auto main() -> int {
    std::cout << "*** Component Tests C++ ***" << std::endl;

    auto node = iox2::NodeBuilder {}.create<iox2::ServiceType::Ipc>().value();
    auto service = node.service_builder(iox2::ServiceName::create("iox2-component-tests").value())
                       .publish_subscribe<ComponentTestHeader>()
                       .open_or_create()
                       .value();
    auto subscriber = service.subscriber_builder().create().value();

    auto tests = component_tests();
    std::cout << "Waiting for clients to connect..." << std::endl;
    int const receive_interval_ms = 100;
    while (service.dynamic_config().number_of_publishers() == 0) {
        if (!node.wait(iox2::bb::Duration::from_millis(receive_interval_ms))) {
            std::cout << "Aborting.\n";
            return 1;
        }
    }
    while (node.wait(iox2::bb::Duration::from_millis(receive_interval_ms)).has_value()) {
        auto sample = subscriber.receive().value();
        if (sample) {
            auto it_test = std::find_if(begin(tests), end(tests), [&sample](Test const& test) -> auto {
                return test.test_name == sample->payload().test_name;
            });
            if (it_test == end(tests)) {
                std::cout << "Unknown component test '" << sample->payload().test_name.unchecked_access().c_str()
                          << "' requested. Aborting." << std::endl;
                return 1;
            }
            std::cout << "   - Running test " << it_test->test_name.unchecked_access().c_str() << "...\n";
            if (!it_test->test->run_test(node)) {
                std::cout << "     Failed.\n";
                return 1;
            }
            std::cout << "     OK.\n";
        } else {
            if (service.dynamic_config().number_of_publishers() == 0) {
                std::cout << "Publisher left. Test complete.\n";
                break;
            }
        }
    }
}
