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

// TODO [#817] see "RAII" in service_types example
int main(void) {
    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_TRACE);

    // create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    iox2_node_h node_handle = NULL;
    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_IPC, &node_handle) != IOX2_OK) {
        printf("Could not create node!\n");
        goto end;
    }

    // create service name
    const char* service_name_value = "My/Funk/ServiceName";
    iox2_service_name_h service_name = NULL;
    if (iox2_service_name_new(NULL, service_name_value, strlen(service_name_value), &service_name) != IOX2_OK) {
        printf("Unable to create service name!\n");
        goto drop_node;
    }

    // create service builder
    iox2_service_name_ptr service_name_ptr = iox2_cast_service_name_ptr(service_name);
    iox2_service_builder_h service_builder = iox2_node_service_builder(&node_handle, NULL, service_name_ptr);
    iox2_service_builder_blackboard_creator_h service_builder_blackboard =
        iox2_service_builder_blackboard_creator(service_builder);

    // set key type
    const char* key_type_name = "Foo";
    if (iox2_service_builder_blackboard_creator_set_key_type_details(
            &service_builder_blackboard, key_type_name, strlen(key_type_name), sizeof(struct Foo), alignof(struct Foo))
        != IOX2_OK) {
        printf("Unable to set key type details!\n");
        goto drop_service_name;
    }

    // set key eq comparison function
    if (iox2_service_builder_blackboard_creator_set_key_eq_comparison_function(&service_builder_blackboard, key_cmp)
        != IOX2_OK) {
        printf("Unable to set the key eq comparison function!\n");
        goto drop_service_name;
    }

    // add key-value pairs
    struct Foo key_0;
    key_0.x = 0;
    key_0.y = -4;
    key_0.z = 4;
    const char* value_type_name_int = "int32_t";
    int32_t value_key_0 = 3;

    iox2_service_builder_blackboard_creator_add(&service_builder_blackboard,
                                                (const uint8_t*) &key_0,
                                                sizeof(struct Foo),
                                                alignof(struct Foo),
                                                &value_key_0,
                                                NULL,
                                                value_type_name_int,
                                                strlen(value_type_name_int),
                                                sizeof(int32_t),
                                                alignof(int32_t));

    struct Foo key_1;
    key_1.x = 1;
    key_1.y = -4;
    key_1.z = 4;
    const char* value_type_name_double = "double";
    const double START_VALUE = 1.1;
    double value_key_1 = START_VALUE;

    iox2_service_builder_blackboard_creator_add(&service_builder_blackboard,
                                                (const uint8_t*) &key_1,
                                                sizeof(struct Foo),
                                                alignof(struct Foo),
                                                &value_key_1,
                                                NULL,
                                                value_type_name_double,
                                                strlen(value_type_name_double),
                                                sizeof(double),
                                                alignof(double));

    // create service
    iox2_port_factory_blackboard_h service = NULL;
    if (iox2_service_builder_blackboard_create(service_builder_blackboard, NULL, &service) != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_name;
    }

    // create writer and entry handles
    iox2_port_factory_writer_builder_h writer_builder = iox2_port_factory_blackboard_writer_builder(&service, NULL);
    iox2_writer_h writer = NULL;
    if (iox2_port_factory_writer_builder_create(writer_builder, NULL, &writer) != IOX2_OK) {
        printf("Unable to create writer!\n");
        goto drop_service;
    }

    iox2_entry_handle_mut_h entry_handle_mut_key_0 = NULL;
    if (iox2_writer_entry(&writer,
                          NULL,
                          &entry_handle_mut_key_0,
                          (const uint8_t*) &key_0,
                          sizeof(struct Foo),
                          alignof(struct Foo),
                          value_type_name_int,
                          strlen(value_type_name_int),
                          sizeof(int32_t),
                          alignof(int32_t))
        != IOX2_OK) {
        printf("Unable to create entry_handle_mut!\n");
        goto drop_writer;
    }

    iox2_entry_handle_mut_h entry_handle_mut_key_1 = NULL;
    if (iox2_writer_entry(&writer,
                          NULL,
                          &entry_handle_mut_key_1,
                          (const uint8_t*) &key_1,
                          sizeof(struct Foo),
                          alignof(struct Foo),
                          value_type_name_double,
                          strlen(value_type_name_double),
                          sizeof(double),
                          alignof(double))
        != IOX2_OK) {
        printf("Unable to create entry_handle_mut!\n");
        goto drop_entry_handle_mut_key_0;
    }

    // update values
    int32_t counter = 0;
    while (iox2_node_wait(&node_handle, 1, 0) == IOX2_OK) {
        counter += 1;

        iox2_entry_handle_mut_update_with_copy(&entry_handle_mut_key_0, &counter, sizeof(int32_t), alignof(int32_t));
        printf("Write value %d for key 0...\n", counter);

        iox2_entry_value_h entry_value = NULL;
        iox2_entry_handle_mut_loan_uninit(entry_handle_mut_key_1, NULL, &entry_value, sizeof(double), alignof(double));
        double* payload = NULL;
        iox2_entry_value_mut(&entry_value, (void**) &payload);
        *payload = START_VALUE * (double) counter;
        iox2_entry_value_update(entry_value, NULL, &entry_handle_mut_key_1);
        printf("Write value %f for key 1...\n", *payload);
    }

    iox2_entry_handle_mut_drop(entry_handle_mut_key_1);

drop_entry_handle_mut_key_0:
    iox2_entry_handle_mut_drop(entry_handle_mut_key_0);

drop_writer:
    iox2_writer_drop(writer);

drop_service:
    iox2_port_factory_blackboard_drop(service);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
