// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

int main(int argc, char** argv) {
    if (argc != 3) {
        printf("usage: %s DOMAIN_NAME SERVICE_NAME\n", argv[0]);
        exit(-1);
    }

    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    // create a new config based on the global config
    iox2_config_ptr config_ptr = iox2_config_global_config();
    iox2_config_h config = NULL;
    iox2_config_from_ptr(config_ptr, NULL, &config);

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    if (iox2_config_global_set_prefix(&config, argv[1]) != IOX2_OK) {
        printf("invalid domain name\"%s\"\n", argv[1]);
        goto drop_config;
    }

    // create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    iox2_node_h node_handle = NULL;

    // use the custom config when creating the custom node
    // every service constructed by the node will use this config
    iox2_node_builder_set_config(&node_builder_handle, &config);
    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_IPC, &node_handle) != IOX2_OK) {
        goto drop_config;
    }

    // create service name
    const char* service_name_value = argv[2];
    iox2_service_name_h service_name = NULL;
    if (iox2_service_name_new(NULL, service_name_value, strlen(service_name_value), &service_name) != IOX2_OK) {
        printf("Unable to create service name!\n");
        goto drop_node;
    }

    // create service builder
    iox2_service_name_ptr service_name_ptr = iox2_cast_service_name_ptr(service_name);
    iox2_service_builder_h service_builder = iox2_node_service_builder(&node_handle, NULL, service_name_ptr);
    iox2_service_builder_pub_sub_h service_builder_pub_sub = iox2_service_builder_pub_sub(service_builder);

    // set pub sub payload type
    const char* payload_type_name = "16TransmissionData";
    if (iox2_service_builder_pub_sub_set_payload_type_details(&service_builder_pub_sub,
                                                              iox2_type_variant_e_FIXED_SIZE,
                                                              payload_type_name,
                                                              strlen(payload_type_name),
                                                              sizeof(struct TransmissionData),
                                                              alignof(struct TransmissionData))
        != IOX2_OK) {
        printf("Unable to set type details\n");
        goto drop_service_name;
    }

    // create service
    iox2_port_factory_pub_sub_h service = NULL;
    if (iox2_service_builder_pub_sub_open_or_create(service_builder_pub_sub, NULL, &service) != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_name;
    }

    // create publisher
    iox2_port_factory_publisher_builder_h publisher_builder =
        iox2_port_factory_pub_sub_publisher_builder(&service, NULL);
    iox2_publisher_h publisher = NULL;
    if (iox2_port_factory_publisher_builder_create(publisher_builder, NULL, &publisher) != IOX2_OK) {
        printf("Unable to create publisher!\n");
        goto drop_service;
    }

    int32_t counter = 0;
    while (iox2_node_wait(&node_handle, 1, 0) == IOX2_OK) {
        counter += 1;

        // loan sample
        iox2_sample_mut_h sample = NULL;
        if (iox2_publisher_loan_slice_uninit(&publisher, NULL, &sample, 1) != IOX2_OK) {
            printf("Failed to loan sample\n");
            goto drop_publisher;
        }

        // write payload
        struct TransmissionData* payload = NULL;
        iox2_sample_mut_payload_mut(&sample, (void**) &payload, NULL);
        payload->x = counter;
        payload->y = counter * 3;
        payload->funky = counter * 812.12; // NOLINT

        // send sample
        if (iox2_sample_mut_send(sample, NULL) != IOX2_OK) {
            printf("Failed to send sample\n");
            goto drop_publisher;
        }

        printf("[domain: \"%s\", service: \"%s\"] Send sample %d ...\n", argv[1], argv[2], counter);
    }

drop_publisher:
    iox2_publisher_drop(publisher);

drop_service:
    iox2_port_factory_pub_sub_drop(service);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

drop_config:
    iox2_config_drop(config);

    //[unused-label] end:
    return 0;
}
