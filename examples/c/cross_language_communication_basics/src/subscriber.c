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
#include "message_data.h"

#if defined(_WIN32) || defined(WIN32) || defined(__WIN32__) || defined(_WIN64)
#define alignof __alignof
#else
#include <stdalign.h>
#endif
#include <stdint.h>
#include <stdio.h>
#include <string.h>

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
    const char* service_name_value = "My/Funk/ServiceName";
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
    if (iox2_service_builder_pub_sub_set_payload_type_details(&service_builder_pub_sub,
                                                              iox2_type_variant_e_FIXED_SIZE,
                                                              IOX2_PAYLOAD_TYPE_NAME,
                                                              strlen(IOX2_PAYLOAD_TYPE_NAME),
                                                              sizeof(struct TransmissionData),
                                                              alignof(struct TransmissionData))
        != IOX2_OK) {
        printf("Unable to set payload type details\n");
        goto drop_service_name;
    }

    // set pub sub user header type
    if (iox2_service_builder_pub_sub_set_user_header_type_details(&service_builder_pub_sub,
                                                                  iox2_type_variant_e_FIXED_SIZE,
                                                                  IOX2_USER_HEADER_TYPE_NAME,
                                                                  strlen(IOX2_USER_HEADER_TYPE_NAME),
                                                                  sizeof(struct CustomHeader),
                                                                  alignof(struct CustomHeader))
        != IOX2_OK) {
        printf("Unable to set user header type details\n");
        goto drop_service_name;
    }

    // create service
    iox2_port_factory_pub_sub_h service = NULL;
    if (iox2_service_builder_pub_sub_open_or_create(service_builder_pub_sub, NULL, &service) != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_name;
    }

    // create subscriber
    iox2_port_factory_subscriber_builder_h subscriber_builder =
        iox2_port_factory_pub_sub_subscriber_builder(&service, NULL);
    iox2_subscriber_h subscriber = NULL;
    if (iox2_port_factory_subscriber_builder_create(subscriber_builder, NULL, &subscriber) != IOX2_OK) {
        printf("Unable to create subscriber!\n");
        goto drop_service;
    }

    printf("Subscriber ready to receive data!\n");

    while (iox2_node_wait(&node_handle, 1, 0) == IOX2_OK) {
        // receive sample
        iox2_sample_h sample = NULL;
        if (iox2_subscriber_receive(&subscriber, NULL, &sample) != IOX2_OK) {
            printf("Failed to receive sample\n");
            goto drop_subscriber;
        }

        if (sample != NULL) {
            struct TransmissionData* payload = NULL;
            iox2_sample_payload(&sample, (const void**) &payload, NULL);

            const struct CustomHeader* user_header = NULL;
            iox2_sample_user_header(&sample, (const void**) &user_header);

            printf(
                "received: TransmissionData { .x: %d, .y: %d, .funky: %.2lf }, user_header: version = %d, timestamp = "
                "%lu\n",
                payload->x,
                payload->y,
                payload->funky,
                user_header->version,
                (long unsigned) user_header->timestamp);
            iox2_sample_drop(sample);
        }
    }


drop_subscriber:
    iox2_subscriber_drop(subscriber);

drop_service:
    iox2_port_factory_pub_sub_drop(service);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
