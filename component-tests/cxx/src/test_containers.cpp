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

#include <iox2/container/static_vector.hpp>

#include <iostream>

namespace {
class ContainerTest : public IComponentTest {
  public:
    ContainerTest() = default;
    ContainerTest(ContainerTest const&) = delete;
    ContainerTest(ContainerTest&&) = delete;
    auto operator=(ContainerTest const&) -> ContainerTest& = delete;
    auto operator=(ContainerTest&&) -> ContainerTest& = delete;
    ~ContainerTest() override = default;
    auto test_name() const -> char const* override {
        return "containers";
    }
    auto run_test(iox2::Node<iox2::ServiceType::Ipc> const& node) -> bool override;
};

struct ContainerTestRequest {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "ContainerTestRequest";
    int32_t vector_type_sequence;
    int32_t container_size;
    int32_t container_alignment;
    int32_t size_of_data_component;
    int32_t offset_of_data_component;
    int32_t size_of_size_component;
    int32_t offset_of_size_component;
    bool size_component_type_is_unsigned;
};

// NOLINTBEGIN(readability-identifier-naming)
// NOLINTNEXTLINE(performance-enum-size)
enum class VectorTypeSequence : int32_t {
    VecI32_10 = 1,
    EndOfTest = -1,
};
// NOLINTEND(readability-identifier-naming)

struct ContainerTestResponse {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "ContainerTestResponse";
    int32_t vector_type_sequence;
    bool all_fields_match;
};

auto check_request(ContainerTestRequest const& req) -> bool {
    switch (req.vector_type_sequence) {
    case static_cast<int32_t>(VectorTypeSequence::VecI32_10): {
        // NOLINTNEXTLINE(readability-magic-numbers,cppcoreguidelines-avoid-magic-numbers)
        iox2::container::StaticVector<int32_t, 10> test_vec;
        auto const stats = test_vec.static_memory_layout_metrics();
        if (static_cast<int32_t>(stats.vector_size) != req.container_size) {
            return false;
        }
        if (static_cast<int32_t>(stats.vector_alignment) != req.container_alignment) {
            return false;
        }
        if (static_cast<int32_t>(stats.storage_metrics.storage_size) != req.container_size) {
            return false;
        }
        if (static_cast<int32_t>(stats.storage_metrics.storage_alignment) != req.container_alignment) {
            return false;
        }
        if (static_cast<int32_t>(stats.storage_metrics.sizeof_bytes) != req.size_of_data_component) {
            return false;
        }
        if (static_cast<int32_t>(stats.storage_metrics.offset_bytes) != req.offset_of_data_component) {
            return false;
        }
        if (static_cast<int32_t>(stats.storage_metrics.sizeof_size) != req.size_of_size_component) {
            return false;
        }
        if (static_cast<int32_t>(stats.storage_metrics.offset_size) != req.offset_of_size_component) {
            return false;
        }
        if (stats.storage_metrics.size_is_unsigned != req.size_component_type_is_unsigned) {
            return false;
        }
    } break;
    case static_cast<int32_t>(VectorTypeSequence::EndOfTest):
        break;
    default:
        return false;
    }
    return true;
}

auto ContainerTest::run_test(iox2::Node<iox2::ServiceType::Ipc> const& node) -> bool {
    auto req_resp = node.service_builder(
                            iox2::ServiceName::create("iox2-component-tests-containers").expect("Invalid service name"))
                        .request_response<ContainerTestRequest, ContainerTestResponse>()
                        .open_or_create()
                        .expect("No request response for test");
    auto server = req_resp.server_builder().create().expect("Unable to create server");
    auto const refresh_interval = iox::units::Duration::fromMilliseconds(100);
    while (req_resp.dynamic_config().number_of_clients() == 0) {
        node.wait(refresh_interval).expect("wait");
    }

    while (node.wait(refresh_interval)) {
        auto receive_request = server.receive();

        if (!receive_request) {
            std::cout << "Error receiving request.\n";
            return false;
        }
        auto& opt_request = receive_request.value();
        if (opt_request) {
            auto& request = opt_request.value();
            std::cout << "       * Processing request " << request.payload().vector_type_sequence << "\n";
            bool const check_succeeded = check_request(request.payload());
            send(request.loan_uninit().expect("").write_payload(
                     ContainerTestResponse { request.payload().vector_type_sequence, check_succeeded }))
                .expect("Response send error");
            if (!check_succeeded) {
                return false;
            }
            if (request.payload().vector_type_sequence == static_cast<int32_t>(VectorTypeSequence::EndOfTest)) {
                return true;
            }
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

auto test_containers() -> std::unique_ptr<IComponentTest> {
    return std::make_unique<ContainerTest>();
}
