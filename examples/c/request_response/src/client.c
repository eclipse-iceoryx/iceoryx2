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

#if defined(_WIN32) || defined(WIN32) || defined(__WIN32__) || defined(_WIN64)
#define alignof __alignof
#else
#include <stdalign.h>
#endif
#include <stdint.h>
#include <stdio.h>
#include <string.h>

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

    // Create client
    iox2_port_factory_client_builder_h client_builder =
        iox2_port_factory_request_response_client_builder(&service, NULL);
    iox2_client_h client = NULL;
    if (iox2_port_factory_client_builder_create(client_builder, NULL, &client) != IOX2_OK) {
        printf("Unable to create client!\n");
        goto drop_service;
    }

    // Start sending requests
    uint64_t request_counter = 0;
    uint64_t response_counter = 0;

    // For the first request, we use the copy API
    printf("send request %d ...\n", (int32_t) request_counter);
    iox2_pending_response_h pending_response = NULL;
    if (iox2_client_send_copy(&client, &request_counter, sizeof(uint64_t), 1, NULL, &pending_response) != IOX2_OK) {
        printf("Failed to send initial request\n");
        goto drop_client;
    }

    // Main loop
    while (iox2_node_wait(&node_handle, 1, 0) == IOX2_OK) {
        // Check for responses
        const struct TransmissionData* response_data = NULL;
        iox2_response_h response = NULL;

        while (true) {
            response = NULL;
            if (iox2_pending_response_receive(&pending_response, NULL, &response) != IOX2_OK) {
                printf("Failed to receive response\n");
                goto drop_client;
            }

            if (response == NULL) {
                break;
            }

            iox2_response_payload(&response, (const void**) &response_data, NULL);
            printf("  received response %d: x=%d, y=%d, funky=%f\n",
                   (int32_t) response_counter,
                   response_data->x,
                   response_data->y,
                   response_data->funky);
            response_counter += 1;
            iox2_response_drop(response);
        }

        request_counter++;

        iox2_pending_response_drop(pending_response);
        pending_response = NULL;

        // For subsequent requests, use the zero-copy API
        printf("send request %d ...\n", (int32_t) request_counter);

        // Loan request sample
        iox2_request_mut_h request = NULL;
        if (iox2_client_loan_slice_uninit(&client, NULL, &request, 1) != IOX2_OK) {
            printf("Failed to loan request\n");
            goto drop_client;
        }

        // Write payload
        uint64_t* payload = NULL;
        iox2_request_mut_payload_mut(&request, (void**) &payload, NULL);
        *payload = request_counter;

        // Send request
        if (iox2_request_mut_send(request, NULL, &pending_response) != IOX2_OK) {
            printf("Failed to send request\n");
            goto drop_client;
        }
    }

    printf("exit\n");

    if (pending_response != NULL) {
        iox2_pending_response_drop(pending_response);
    }

drop_client:
    iox2_client_drop(client);

drop_service:
    iox2_port_factory_request_response_drop(service);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
