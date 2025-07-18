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

#include "iox2/iceoryx2.h"
#include "transmission_data.h"

#ifdef _WIN64
#define alignof __alignof
#else
#include <stdalign.h>
#endif
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define MILLISECOND_IN_NS 100000           // NOLINT
#define CYCLE_TIME 100 * MILLISECOND_IN_NS // NOLINT

int main(void) { // NOLINT
    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    // Create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    iox2_node_h node_handle = NULL;
    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_IPC, &node_handle) != IOX2_OK) {
        printf("Could not create node!\n");
        goto end;
    }

    // Create service name
    const char* service_name_value = "My/Funk/ServiceName";
    iox2_service_name_h service_name = NULL;
    if (iox2_service_name_new(NULL, service_name_value, strlen(service_name_value), &service_name) != IOX2_OK) {
        printf("Unable to create service name!\n");
        goto drop_node;
    }

    // Create service builder
    iox2_service_name_ptr service_name_ptr = iox2_cast_service_name_ptr(service_name);
    iox2_service_builder_h service_builder = iox2_node_service_builder(&node_handle, NULL, service_name_ptr);
    iox2_service_builder_request_response_h service_builder_request_response =
        iox2_service_builder_request_response(service_builder);

    // Set request and response type details
    const char* request_type_name = "u64";
    const char* response_type_name = "TransmissionData";

    if (iox2_service_builder_request_response_set_request_payload_type_details(&service_builder_request_response,
                                                                               iox2_type_variant_e_FIXED_SIZE,
                                                                               request_type_name,
                                                                               strlen(request_type_name),
                                                                               sizeof(uint64_t),
                                                                               alignof(uint64_t))
        != IOX2_OK) {
        printf("Unable to set request type details\n");
        goto drop_service_name;
    }

    if (iox2_service_builder_request_response_set_response_payload_type_details(&service_builder_request_response,
                                                                                iox2_type_variant_e_FIXED_SIZE,
                                                                                response_type_name,
                                                                                strlen(response_type_name),
                                                                                sizeof(struct TransmissionData),
                                                                                alignof(struct TransmissionData))
        != IOX2_OK) {
        printf("Unable to set response type details\n");
        goto drop_service_name;
    }

    // Create service
    iox2_port_factory_request_response_h service = NULL;
    if (iox2_service_builder_request_response_open_or_create(service_builder_request_response, NULL, &service)
        != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_name;
    }

    // Create server
    iox2_port_factory_server_builder_h server_builder =
        iox2_port_factory_request_response_server_builder(&service, NULL);
    iox2_server_h server = NULL;
    if (iox2_port_factory_server_builder_create(server_builder, NULL, &server) != IOX2_OK) {
        printf("Unable to create server!\n");
        goto drop_service;
    }

    printf("Server ready to receive requests!\n");

    // Main loop
    int32_t counter = 0;
    while (iox2_node_wait(&node_handle, 0, CYCLE_TIME) == IOX2_OK) { // 100ms in nanoseconds
        // Receive requests
        iox2_active_request_h active_request = NULL;

        while (true) {
            active_request = NULL;
            if (iox2_server_receive(&server, NULL, &active_request) != IOX2_OK) {
                printf("Failed to receive request\n");
                goto drop_server;
            }

            if (active_request == NULL) {
                break;
            }

            // Get request payload
            uint64_t* request_value = NULL;
            iox2_active_request_payload(&active_request, (const void**) &request_value, NULL);

            printf("received request: %d\n", (int32_t) *request_value);

            // Create response data
            struct TransmissionData response = { .x = 5 + counter, .y = 6 * counter, .funky = 7.77 }; // NOLINT

            printf("  send response: x=%d, y=%d, funky=%f\n", response.x, response.y, response.funky);

            // Send first response using copy API
            if (iox2_active_request_send_copy(&active_request, &response, sizeof(struct TransmissionData), 1)
                != IOX2_OK) {
                printf("Failed to send response\n");
                continue;
            }

            // Optionally send additional responses using zero-copy API (mimicking the Rust example's behavior)
            for (int32_t iter = 0; iter < (int32_t) (*request_value % 2); iter++) {
                iox2_response_mut_h response = NULL;
                if (iox2_active_request_loan_slice_uninit(&active_request, NULL, &response, 1) != IOX2_OK) {
                    printf("Failed to loan response sample\n");
                    continue;
                }

                // Write payload
                struct TransmissionData* payload = NULL;
                iox2_response_mut_payload_mut(&response, (void**) &payload, NULL);

                payload->x = counter * (iter + 1);
                payload->y = counter + iter;
                payload->funky = counter * 0.1234; // NOLINT

                printf("  send response: x=%d, y=%d, funky=%f\n", payload->x, payload->y, payload->funky);

                // Send response
                if (iox2_response_mut_send(response) != IOX2_OK) {
                    printf("Failed to send additional response\n");
                }
            }

            // Drop the active request when done with it
            iox2_active_request_drop(active_request);
        }

        counter++;
    }

    printf("exit\n");

drop_server:
    iox2_server_drop(server);

drop_service:
    iox2_port_factory_request_response_drop(service);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
