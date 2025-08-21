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

#ifdef _WIN64
#define alignof __alignof
#else
#include <stdalign.h>
#endif
#include <stdint.h>
#include <stdio.h>
#include <string.h>

void release_callback(void* value_ptr) {
    free(value_ptr);
}

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
    iox2_service_builder_blackboard_opener_h service_builder_blackboard =
        iox2_service_builder_blackboard_opener(service_builder);

    // set key type
    const char* value_type_name = "uint64_t";
    if (iox2_service_builder_blackboard_opener_set_key_type_details(
            &service_builder_blackboard, value_type_name, strlen(value_type_name), sizeof(uint64_t), alignof(uint64_t))
        != IOX2_OK) {
        printf("Unable to set key type details!\n");
        goto drop_service_name;
    }

    // create service
    iox2_port_factory_blackboard_h service = NULL;
    if (iox2_service_builder_blackboard_open(service_builder_blackboard, NULL, &service) != IOX2_OK) {
        printf("Unable to open service!\n");
        goto drop_service_name;
    }

    // create reader and entry handles
    iox2_port_factory_reader_builder_h reader_builder = iox2_port_factory_blackboard_reader_builder(&service, NULL);
    iox2_reader_h reader = NULL;
    if (iox2_port_factory_reader_builder_create(reader_builder, NULL, &reader) != IOX2_OK) {
        printf("Unable to create reader!\n");
        goto drop_service;
    }

    iox2_entry_handle_h entry_handle_key_0 = NULL;
    if (iox2_reader_entry(&reader,
                          NULL,
                          &entry_handle_key_0,
                          0,
                          value_type_name,
                          strlen(value_type_name),
                          sizeof(uint64_t),
                          alignof(uint64_t))
        != IOX2_OK) {
        printf("Unable to create entry_handle!\n");
        goto drop_reader;
    }

    const char* value_type_name_float = "float";
    iox2_entry_handle_h entry_handle_key_1 = NULL;
    if (iox2_reader_entry(&reader,
                          NULL,
                          &entry_handle_key_1,
                          1,
                          value_type_name_float,
                          strlen(value_type_name_float),
                          sizeof(float),
                          alignof(float))
        != IOX2_OK) {
        printf("Unable to create entry_handle!\n");
        goto drop_entry_handle_key_0;
    }

    uint64_t value_0 = 0;
    float value_1 = 0.0;
    while (iox2_node_wait(&node_handle, 1, 0) == IOX2_OK) {
        iox2_entry_handle_get(&entry_handle_key_0, &value_0, sizeof(uint64_t), alignof(uint64_t));
        printf("Read value %lu for key 0...\n", value_0);

        iox2_entry_handle_get(&entry_handle_key_1, &value_1, sizeof(float), alignof(float));
        printf("Read value %f for key 1 ...\n", value_1);
    }

    iox2_entry_handle_drop(entry_handle_key_1);

drop_entry_handle_key_0:
    iox2_entry_handle_drop(entry_handle_key_0);

drop_reader:
    iox2_reader_drop(reader);

drop_service:
    iox2_port_factory_blackboard_drop(service);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
