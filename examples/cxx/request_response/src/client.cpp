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

#include "iox2/config.hpp"
#include "iox2/iceoryx2.hpp"
#include "transmission_data.hpp"
#include <cstdint>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_millis(20);

const char* RequestResponseOpenOrCreateErrorString[] = {
    "OpenDoesNotExist",
    "OpenDoesNotSupportRequestedAmountOfClientRequestLoans",
    "OpenDoesNotSupportRequestedAmountOfActiveRequestsPerClient",
    "OpenDoesNotSupportRequestedResponseBufferSize",
    "OpenDoesNotSupportRequestedAmountOfServers",
    "OpenDoesNotSupportRequestedAmountOfClients",
    "OpenDoesNotSupportRequestedAmountOfNodes",
    "OpenDoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse",
    "OpenExceedsMaxNumberOfNodes",
    "OpenHangsInCreation",
    "OpenIncompatibleRequestType",
    "OpenIncompatibleResponseType",
    "OpenIncompatibleAttributes",
    "OpenIncompatibleMessagingPattern",
    "OpenIncompatibleOverflowBehaviorForRequests",
    "OpenIncompatibleOverflowBehaviorForResponses",
    "OpenIncompatibleBehaviorForFireAndForgetRequests",
    "OpenInsufficientPermissions",
    "OpenInternalFailure",
    "OpenIsMarkedForDestruction",
    "OpenServiceInCorruptedState",
    "CreateAlreadyExists",
    "CreateInternalFailure",
    "CreateIsBeingCreatedByAnotherInstance",
    "CreateInsufficientPermissions",
    "CreateHangsInCreation",
    "CreateServiceInCorruptedState",
    "SystemInFlux",
};

const char* ClientCreateErrorStrings[] = {
    "UnableToCreateDataSegment",
    "ExceedsMaxSupportedClients",
    "FailedToDeployThreadsafetyPolicy",
};

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);

    auto config = Config::global_config().to_owned();
    config.global().node().set_cleanup_dead_nodes_on_creation(true);
    config.global().node().set_cleanup_dead_nodes_on_destruction(false);

    auto node = NodeBuilder().config(config).create<ServiceType::Ipc>().value();

    auto service_result = node.service_builder(ServiceName::create("My/Funk/ServiceName").value())
                              .request_response<uint64_t, TransmissionData>()
                              .max_servers(2)
                              .max_clients(1)
                              .open_or_create();
    while (!service_result.has_value()
           && (service_result.error() == RequestResponseOpenOrCreateError::OpenHangsInCreation
               || service_result.error() == RequestResponseOpenOrCreateError::CreateHangsInCreation)) {
        service_result = node.service_builder(ServiceName::create("My/Funk/ServiceName").value())
                             .request_response<uint64_t, TransmissionData>()
                             .max_servers(2)
                             .max_clients(1)
                             .open_or_create();
    }

    if (!service_result.has_value()) {
        auto error_index = static_cast<uint64_t>(service_result.error());
        std::cout << "#### service open or create error: [" << error_index << "] "
                  << RequestResponseOpenOrCreateErrorString[error_index] << std::endl;
    }
    auto service = std::move(service_result).value();

    auto client_result = service.client_builder().create();
    while (!client_result.has_value() && client_result.error() == ClientCreateError::ExceedsMaxSupportedClients) {
        client_result = service.client_builder().create();
    }

    if (!client_result.has_value()) {
        auto error_index = static_cast<uint64_t>(client_result.error());
        std::cout << "#### client create error: [" << error_index << "] " << ClientCreateErrorStrings[error_index]
                  << std::endl;
    }
    auto client = std::move(client_result).value();

    auto request_counter = 0U;
    auto response_counter = 0U;

    // sending first request by using slower, inefficient copy API
    std::cout << "send request " << request_counter << " ..." << std::endl;
    auto pending_response = client.send_copy(request_counter).value();

    while (node.wait(CYCLE_TIME).has_value()) {
        // acquire all responses to our request from our buffer that were sent by the servers
        while (true) {
            auto response = pending_response.receive().value();
            if (response.has_value()) {
                std::cout << "received response " << response_counter << ": " << response->payload() << std::endl;
                response_counter += 1;
            } else {
                break;
            }
        }

        request_counter += 1;
        // send all other requests by using zero copy API
        auto request = client.loan_uninit().value();
        auto initialized_request = request.write_payload(request_counter);

        pending_response = send(std::move(initialized_request)).value();

        std::cout << "send request " << request_counter << " ..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
