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

#ifndef IOX2_INCLUDE_GUARD_COMPONENT_TESTS_COMMON_HPP
#define IOX2_INCLUDE_GUARD_COMPONENT_TESTS_COMMON_HPP

#include <iox2/iceoryx2.hpp>

#include <memory>

class IComponentTest {
  protected:
    IComponentTest() = default;

  public:
    virtual ~IComponentTest() = default;
    IComponentTest(IComponentTest const&) = delete;
    IComponentTest(IComponentTest&&) = delete;
    auto operator=(IComponentTest const&) -> IComponentTest& = delete;
    auto operator=(IComponentTest&&) -> IComponentTest& = delete;

    virtual auto test_name() const -> char const* = 0;
    virtual auto run_test(iox2::Node<iox2::ServiceType::Ipc> const& node) -> bool = 0;
};

auto test_containers() -> std::unique_ptr<IComponentTest>;
auto test_container_mutation() -> std::unique_ptr<IComponentTest>;


template <typename RequestType, typename ResponseType>
struct RequestResponseServer {
    iox2::PortFactoryRequestResponse<iox2::ServiceType::Ipc, RequestType, void, ResponseType, void> request_response;
    iox2::Server<iox2::ServiceType::Ipc, RequestType, void, ResponseType, void> server;
};

template <typename RequestType, typename ResponseType>
auto create_server(iox2::Node<iox2::ServiceType::Ipc> const& node,
                   char const* service_name,
                   iox2::legacy::units::Duration const& refresh_interval)
    -> iox2::container::Optional<RequestResponseServer<RequestType, ResponseType>> {
    auto exp_service_name = iox2::ServiceName::create(service_name);
    if (!exp_service_name) {
        std::cout << "Error creating service name\n";
        return iox2::container::nullopt;
    }
    auto exp_req_resp =
        node.service_builder(exp_service_name.value()).request_response<RequestType, ResponseType>().open_or_create();
    if (!exp_req_resp) {
        std::cout << "Error creating request response for test\n";
        return iox2::container::nullopt;
    }
    auto req_resp = std::move(exp_req_resp.value());
    auto exp_server = req_resp.server_builder().create();
    if (!exp_server) {
        std::cout << "Unable to create request response server\n";
        return iox2::container::nullopt;
    }
    auto server = std::move(exp_server.value());
    while (req_resp.dynamic_config().number_of_clients() == 0) {
        if (!node.wait(refresh_interval)) {
            return iox2::container::nullopt;
        }
    }
    return RequestResponseServer<RequestType, ResponseType> { std::move(req_resp), std::move(server) };
}

#endif
