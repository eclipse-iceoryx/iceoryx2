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

#if defined(_WIN32) || defined(WIN32) || defined(__WIN32__) || defined(_WIN64)
#define alignof __alignof
#else
#include <stdalign.h>
#endif
#include <stdint.h>
#include <stdio.h>
#include <string.h>

enum {
    CycleTime = 750000000
};

struct res { // NOLINT
    iox2_node_h node;
    iox2_service_name_h service_name;
    iox2_port_factory_pub_sub_h service;
    iox2_publisher_h publisher;
};

void init_res(struct res* const value) { // NOLINT
    value->node = NULL;
    value->service_name = NULL;
    value->service = NULL;
    value->publisher = NULL;
}

void drop_res(struct res* const value) { // NOLINT
    if (value->publisher != NULL) {
        iox2_publisher_drop(value->publisher);
    }

    if (value->service != NULL) {
        iox2_port_factory_pub_sub_drop(value->service);
    }

    if (value->service_name != NULL) {
        iox2_service_name_drop(value->service_name);
    }

    if (value->node != NULL) {
        iox2_node_drop(value->node);
    }
}

int main(void) {
    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    struct res example;
    init_res(&example);

    // create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    // The third argument defines the service variant. Different variants can use
    // different mechanisms. For instance the upcoming `iox2_service_type_e_CUDA` would use GPU
    // memory or the `iox2_service_type_e_IPC` would use mechanisms that are optimized for
    // intra-process communication.
    //
    // All services which are created via this `Node` use the same service variant.
    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_IPC, &example.node) != IOX2_OK) {
        printf("Could not create node!\n");
        goto end;
    }

    // create service name
    const char* service_name_value = "Service-Variants-Example";
    if (iox2_service_name_new(NULL, service_name_value, strlen(service_name_value), &example.service_name) != IOX2_OK) {
        printf("Unable to create service name!\n");
        goto end;
    }

    // create service builder
    iox2_service_name_ptr service_name_ptr = iox2_cast_service_name_ptr(example.service_name);
    iox2_service_builder_h service_builder = iox2_node_service_builder(&example.node, NULL, service_name_ptr);
    iox2_service_builder_pub_sub_h service_builder_pub_sub = iox2_service_builder_pub_sub(service_builder);

    // set pub sub payload type
    const char* payload_type_name = "u64";
    if (iox2_service_builder_pub_sub_set_payload_type_details(&service_builder_pub_sub,
                                                              iox2_type_variant_e_FIXED_SIZE,
                                                              payload_type_name,
                                                              strlen(payload_type_name),
                                                              sizeof(uint64_t),
                                                              alignof(uint64_t))
        != IOX2_OK) {
        printf("Unable to set type details\n");
        goto end;
    }

    // create service
    if (iox2_service_builder_pub_sub_open_or_create(service_builder_pub_sub, NULL, &example.service) != IOX2_OK) {
        printf("Unable to create service!\n");
        goto end;
    }

    // create publisher
    iox2_port_factory_publisher_builder_h publisher_builder =
        iox2_port_factory_pub_sub_publisher_builder(&example.service, NULL);
    if (iox2_port_factory_publisher_builder_create(publisher_builder, NULL, &example.publisher) != IOX2_OK) {
        printf("Unable to create publisher!\n");
        goto end;
    }

    uint64_t counter = 0;
    while (iox2_node_wait(&example.node, 0, CycleTime) == IOX2_OK) {
        printf("send: %llu\n", (unsigned long long) counter);
        if (iox2_publisher_send_copy(&example.publisher, (void*) &counter, sizeof(counter), NULL) != IOX2_OK) {
            printf("Failed to send sample\n");
            goto end;
        }
        counter += 1;
    }

end:
    drop_res(&example);
    return 0;
}
