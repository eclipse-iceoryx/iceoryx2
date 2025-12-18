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

#include <iox2/bb/optional.hpp>
#include <iox2/bb/static_vector.hpp>

#include <iostream>

namespace {
class ContainerMutationTest : public IComponentTest {
  public:
    ContainerMutationTest() = default;
    ContainerMutationTest(ContainerMutationTest const&) = delete;
    ContainerMutationTest(ContainerMutationTest&&) = delete;
    auto operator=(ContainerMutationTest const&) -> ContainerMutationTest& = delete;
    auto operator=(ContainerMutationTest&&) -> ContainerMutationTest& = delete;
    ~ContainerMutationTest() override = default;
    auto test_name() const -> char const* override {
        return "container_mutation";
    }
    auto run_test(iox2::Node<iox2::ServiceType::Ipc> const& node) -> bool override;
};

// NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
struct ContainerMutationTestRequest {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "ContainerMutationTestRequest";
    iox2::bb::StaticVector<int32_t, 10> vector_add_element;
    iox2::bb::StaticVector<int32_t, 10> vector_remove_element;
    iox2::bb::StaticString<64> string_append;
    iox2::bb::StaticVector<iox2::bb::StaticString<16>, 5> vector_strings_change_middle;
};

struct ContainerMutationTestResponse {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "ContainerMutationTestResponse";
    iox2::bb::StaticVector<int32_t, 10> vector_add_element;
    iox2::bb::StaticVector<int32_t, 10> vector_remove_element;
    iox2::bb::StaticString<64> string_append;
    iox2::bb::StaticVector<iox2::bb::StaticString<16>, 5> vector_strings_change_middle;
};

auto check_request(ContainerMutationTestRequest const& req) -> bool {
    if (req.vector_add_element != iox2::bb::StaticVector<int32_t, 10>({ 1, 2, 3, 4 })) {
        return false;
    }
    if (req.vector_remove_element != iox2::bb::StaticVector<int32_t, 10>({ 1, 2, 9999, 3, 4, 9999, 5, 9999 })) {
        return false;
    }
    if (req.string_append != *iox2::bb::StaticString<64>::from_utf8("Hello")) {
        return false;
    }
    if (req.vector_strings_change_middle
        != iox2::bb::StaticVector<iox2::bb::StaticString<16>, 5>(
            { *iox2::bb::StaticString<16>::from_utf8("Howdy!"),
              *iox2::bb::StaticString<16>::from_utf8("Yeehaw!"),
              *iox2::bb::StaticString<16>::from_utf8("How's the missus"),
              *iox2::bb::StaticString<16>::from_utf8("I'll be gone"),
              *iox2::bb::StaticString<16>::from_utf8("See you soon") })) {
        return false;
    }
    return true;
}
// NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)

auto prepare_response(ContainerMutationTestRequest const& request)
    -> iox2::bb::Optional<ContainerMutationTestResponse> {
    // NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    ContainerMutationTestResponse response;
    response.vector_add_element = request.vector_add_element;
    response.vector_add_element.try_push_back(123);
    response.vector_remove_element = request.vector_remove_element;
    if (!(response.vector_remove_element.try_erase_at(5) && response.vector_remove_element.try_erase_at(2)
          && response.vector_remove_element.try_pop_back())) {
        return iox2::bb::NULLOPT;
    }
    response.string_append = request.string_append;
    response.string_append.try_append_utf8_null_terminated_unchecked(" my baby, hello my honey, hello my ragtime gal");
    response.vector_strings_change_middle = request.vector_strings_change_middle;
    if (!(response.vector_strings_change_middle.element_at(2)->get().unchecked_code_units().try_erase_at(13, 16)
          && response.vector_strings_change_middle.element_at(2)->get().try_append_utf8_null_terminated_unchecked(
              "ter"))) {
        return iox2::bb::NULLOPT;
    }
    // NOLINTEND(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
    return response;
}

auto ContainerMutationTest::run_test(iox2::Node<iox2::ServiceType::Ipc> const& node) -> bool {
    auto const refresh_interval = iox2::bb::Duration::from_millis(100);
    auto opt_server = create_server<ContainerMutationTestRequest, ContainerMutationTestResponse>(
        node, "iox2-component-tests-container_mutation", refresh_interval);
    if (!opt_server) {
        return false;
    }
    auto& req_resp = opt_server->request_response;
    auto& server = opt_server->server;

    while (node.wait(refresh_interval)) {
        auto receive_request = server.receive();

        if (!receive_request) {
            std::cout << "Error receiving request.\n";
            return false;
        }
        auto& opt_request = receive_request.value();
        if (opt_request) {
            auto& request = opt_request.value();
            if (!check_request(request.payload())) {
                return false;
            }
            auto opt_response = prepare_response(request.payload());
            if (!opt_response) {
                return false;
            }
            ContainerMutationTestResponse& response = *opt_response;
            auto exp_response = request.loan_uninit();
            if (!exp_response) {
                std::cout << "Error loaning response\n";
                return false;
            }
            if (!send(exp_response.value().write_payload(std::move(response)))) {
                std::cout << "Error sending response\n";
                return false;
            }
            return true;
        } else {
            if (req_resp.dynamic_config().number_of_clients() == 0) {
                std::cout << "Unexpectedly lost connection with client.\n";
                return false;
            }
        }
    }
    return false;
}
} // namespace

auto test_container_mutation() -> std::unique_ptr<IComponentTest> {
    return std::make_unique<ContainerMutationTest>();
}
