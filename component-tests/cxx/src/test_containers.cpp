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
    VecI64_20 = 2,
    VecOverAligned_5 = 3,
    VecVec8_10 = 4,
    EndOfTest = -1,
};
// NOLINTEND(readability-identifier-naming)

struct ContainerTestResponse {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "ContainerTestResponse";
    int32_t vector_type_sequence;
    bool all_fields_match;
};

// NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers)
struct alignas(64) ContainerTestOverAligned {
    int32_t i;
};

template <typename TestType, uint64_t TestCapacity>
auto check_metrics_for_vector(ContainerTestRequest const& req) -> bool {
    iox2::container::StaticVector<TestType, TestCapacity> test_vec;
    auto const stats = test_vec.static_memory_layout_metrics();
    if ((stats.vector_size > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.vector_size) != req.container_size)) {
        std::cout << "Container size mismatch\n";
        return false;
    }
    if ((stats.vector_alignment > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.vector_alignment) != req.container_alignment)) {
        std::cout << "Container alignment mismatch\n";
        return false;
    }
    if ((stats.storage_metrics.storage_size > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.storage_metrics.storage_size) != req.container_size)) {
        std::cout << "Storage size mismatch\n";
        return false;
    }
    if ((stats.storage_metrics.storage_alignment > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.storage_metrics.storage_alignment) != req.container_alignment)) {
        std::cout << "Storage alignmnent mismatch\n";
        return false;
    }
    if ((stats.storage_metrics.sizeof_bytes > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.storage_metrics.sizeof_bytes) != req.size_of_data_component)) {
        std::cout << "Storage data size mismatch\n";
        return false;
    }
    if ((stats.storage_metrics.offset_bytes > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.storage_metrics.offset_bytes) != req.offset_of_data_component)) {
        std::cout << "Storage data offset mismatch\n";
        return false;
    }
    if ((stats.storage_metrics.sizeof_size > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.storage_metrics.sizeof_size) != req.size_of_size_component)) {
        std::cout << "Storage size size mismatch\n";
        return false;
    }
    if ((stats.storage_metrics.offset_size > std::numeric_limits<int32_t>::max())
        || (static_cast<int32_t>(stats.storage_metrics.offset_size) != req.offset_of_size_component)) {
        std::cout << "Storage size offset mismatch\n";
        return false;
    }
    if (stats.storage_metrics.size_is_unsigned != req.size_component_type_is_unsigned) {
        std::cout << "Storage size signedness mismatch\n";
        return false;
    }
    return true;
}

auto check_request(ContainerTestRequest const& req) -> bool {
    switch (req.vector_type_sequence) {
    case static_cast<int32_t>(VectorTypeSequence::VecI32_10):
        // NOLINTNEXTLINE(readability-magic-numbers,cppcoreguidelines-avoid-magic-numbers)
        return check_metrics_for_vector<int32_t, 10>(req);
    case static_cast<int32_t>(VectorTypeSequence::VecI64_20):
        // NOLINTNEXTLINE(readability-magic-numbers,cppcoreguidelines-avoid-magic-numbers)
        return check_metrics_for_vector<int64_t, 20>(req);
    case static_cast<int32_t>(VectorTypeSequence::VecOverAligned_5):
        // NOLINTNEXTLINE(readability-magic-numbers,cppcoreguidelines-avoid-magic-numbers)
        return check_metrics_for_vector<ContainerTestOverAligned, 5>(req);
    case static_cast<int32_t>(VectorTypeSequence::VecVec8_10):
        // NOLINTNEXTLINE(readability-magic-numbers,cppcoreguidelines-avoid-magic-numbers)
        return check_metrics_for_vector<iox2::container::StaticVector<int8_t, 10>, 10>(req);
    case static_cast<int32_t>(VectorTypeSequence::EndOfTest):
        break;
    default:
        std::cout << "Unknown request type " << req.vector_type_sequence << "\n";
        return false;
    }
    return true;
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity,readability-function-size)
auto ContainerTest::run_test(iox2::Node<iox2::ServiceType::Ipc> const& node) -> bool {
    auto exp_service_name = iox2::ServiceName::create("iox2-component-tests-containers");
    if (!exp_service_name) {
        std::cout << "Error creating service name\n";
        return false;
    }
    auto exp_req_resp = node.service_builder(exp_service_name.value())
                            .request_response<ContainerTestRequest, ContainerTestResponse>()
                            .open_or_create();
    if (!exp_req_resp) {
        std::cout << "Error creating request response for test\n";
        return false;
    }
    auto& req_resp = exp_req_resp.value();
    auto exp_server = req_resp.server_builder().create();
    if (!exp_server) {
        std::cout << "Unable to create request response server\n";
        return false;
    }
    auto& server = exp_server.value();
    auto const refresh_interval = iox::units::Duration::fromMilliseconds(100);
    while (req_resp.dynamic_config().number_of_clients() == 0) {
        if (!node.wait(refresh_interval)) {
            return false;
        }
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
            auto exp_response = request.loan_uninit();
            if (!exp_response) {
                std::cout << "Error loaning response\n";
                return false;
            }
            auto& response = exp_response.value();
            auto exp_send_result = send(response.write_payload(
                ContainerTestResponse { request.payload().vector_type_sequence, check_succeeded }));
            if (!exp_send_result) {
                std::cout << "Error sending response\n";
                return false;
            }
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
