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

    // create the service attribute specifier
    iox2_attribute_specifier_h attribute_specifier = NULL;
    if (iox2_attribute_specifier_new(NULL, &attribute_specifier) != IOX2_OK) {
        printf("Unable to create service attribute specifier");
        goto drop_service_name;
    }

    iox2_attribute_specifier_define(&attribute_specifier, "dds_service_mapping", "my_funky_service_name");
    iox2_attribute_specifier_define(&attribute_specifier, "tcp_serialization_format", "cdr");
    iox2_attribute_specifier_define(&attribute_specifier, "someip_service_mapping", "1/2/3");
    iox2_attribute_specifier_define(&attribute_specifier, "camera_resolution", "1920x1080");

    // create service
    iox2_port_factory_pub_sub_h service = NULL;
    if (iox2_service_builder_pub_sub_create_with_attributes(
            service_builder_pub_sub, &attribute_specifier, NULL, &service)
        != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_attribute_specifier;
    }

    // create publisher
    iox2_port_factory_publisher_builder_h publisher_builder =
        iox2_port_factory_pub_sub_publisher_builder(&service, NULL);
    iox2_publisher_h publisher = NULL;
    if (iox2_port_factory_publisher_builder_create(publisher_builder, NULL, &publisher) != IOX2_OK) {
        printf("Unable to create publisher!\n");
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
        // loan sample
        iox2_sample_mut_h sample = NULL;
        if (iox2_publisher_loan_slice_uninit(&publisher, NULL, &sample, 1) != IOX2_OK) {
            printf("Failed to loan sample\n");
            goto drop_publisher;
        }

        // write payload
        uint64_t* payload = NULL;
        iox2_sample_mut_payload_mut(&sample, (void**) &payload, NULL);
        *payload = 0;

        // send sample
        if (iox2_sample_mut_send(sample, NULL) != IOX2_OK) {
            printf("Failed to send sample\n");
            goto drop_publisher;
        }
    }


drop_publisher:
    iox2_publisher_drop(publisher);

drop_service:
    iox2_port_factory_pub_sub_drop(service);

drop_service_attribute_specifier:
    iox2_attribute_specifier_drop(attribute_specifier);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
