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

enum {
    AttributeBufferSize = 256
};

int main(void) {
    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    // create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    iox2_node_h node_handle = NULL;
    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_IPC, &node_handle) != IOX2_OK) {
        printf("Could not create node!\n");
        goto end;
    }

    // create service name
    const char* service_name_value = "Service/With/Properties";
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
    const char* payload_type_name = "u64";
    if (iox2_service_builder_pub_sub_set_payload_type_details(&service_builder_pub_sub,
                                                              iox2_type_variant_e_FIXED_SIZE,
                                                              payload_type_name,
                                                              strlen(payload_type_name),
                                                              sizeof(uint64_t),
                                                              alignof(uint64_t))
        != IOX2_OK) {
        printf("Unable to set type details\n");
        goto drop_service_name;
    }

    // create the service attribute verifier
    iox2_attribute_verifier_h attribute_verifier = NULL;
    if (iox2_attribute_verifier_new(NULL, &attribute_verifier) != IOX2_OK) {
        printf("Unable to create service attribute verifier");
        goto drop_service_name;
    }

    iox2_attribute_verifier_require(&attribute_verifier, "camera_resolution", "1920x1080");
    iox2_attribute_verifier_require_key(&attribute_verifier, "dds_service_mapping");

    // create service
    iox2_port_factory_pub_sub_h service = NULL;
    if (iox2_service_builder_pub_sub_open_with_attributes(service_builder_pub_sub, &attribute_verifier, NULL, &service)
        != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_attribute_verifier;
    }

    // create subscriber
    iox2_port_factory_subscriber_builder_h subscriber_builder =
        iox2_port_factory_pub_sub_subscriber_builder(&service, NULL);
    iox2_subscriber_h subscriber = NULL;
    if (iox2_port_factory_subscriber_builder_create(subscriber_builder, NULL, &subscriber) != IOX2_OK) {
        printf("Unable to create subscriber!\n");
        goto drop_service;
    }

    // print attributes
    iox2_attribute_set_ptr attribute_set_ptr = iox2_port_factory_pub_sub_attributes(&service);
    size_t number_of_attributes = iox2_attribute_set_number_of_attributes(attribute_set_ptr);
    printf("defined service attributes:");
    for (size_t i = 0; i < number_of_attributes; ++i) {
        iox2_attribute_h_ref attribute_ref = iox2_attribute_set_index(attribute_set_ptr, i);
        char buffer[AttributeBufferSize];
        iox2_attribute_key(attribute_ref, &buffer[0], AttributeBufferSize);
        printf(" Attribute { key: \"%s,", buffer);
        iox2_attribute_value(attribute_ref, &buffer[0], AttributeBufferSize);
        printf(" value: \"%s }", buffer);
    }
    printf("\n");

    while (iox2_node_wait(&node_handle, 1, 0) == IOX2_OK) {
        // receive sample
        iox2_sample_h sample = NULL;
        if (iox2_subscriber_receive(&subscriber, NULL, &sample) != IOX2_OK) {
            printf("Failed to receive sample\n");
            goto drop_subscriber;
        }

        if (sample != NULL) {
            uint64_t* payload = NULL;
            iox2_sample_payload(&sample, (const void**) &payload, NULL);

            printf("received: %d\n", (int) *payload);
            iox2_sample_drop(sample);
        }
    }


drop_subscriber:
    iox2_subscriber_drop(subscriber);

drop_service:
    iox2_port_factory_pub_sub_drop(service);

drop_service_attribute_verifier:
    iox2_attribute_verifier_drop(attribute_verifier);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
