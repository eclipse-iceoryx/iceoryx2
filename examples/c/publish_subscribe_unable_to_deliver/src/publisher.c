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

// NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers) fine for examples

// NOLINTNEXTLINE
struct callback_context_t {
    int fail_counter;
};

static iox2_unable_to_deliver_action_e unable_to_deliver_handler(iox2_unable_to_deliver_info_h_ref info,
                                                                 iox2_callback_context ctx) {
    iox2_buffer_16_align_4_t buf;
    printf("Discarded sample from publisher sender id 0x");
    iox2_unable_to_deliver_info_sender_port_id(info, &buf);
    for (int i = 0; i < 16; ++i) {
        printf("%02X", buf.data[i]);
    }
    printf(" to subscriber receiver id 0x");
    iox2_unable_to_deliver_info_receiver_port_id(info, &buf);
    for (int i = 0; i < 16; ++i) {
        printf("%02X", buf.data[i]);
    }
    printf("\n");
    if (ctx) {
        struct callback_context_t* callback_ctx = (struct callback_context_t*) ctx;
        (*callback_ctx).fail_counter += 1;
        printf("Fail counter: %i\n", (*callback_ctx).fail_counter);
    }
    return iox2_unable_to_deliver_action_e_DISCARD_DATA_AND_FAIL;
}

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
    const char* payload_type_name = "TransmissionData";
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

    iox2_service_builder_pub_sub_set_enable_safe_overflow(&service_builder_pub_sub, false);

    // create service
    iox2_port_factory_pub_sub_h service = NULL;
    if (iox2_service_builder_pub_sub_open_or_create(service_builder_pub_sub, NULL, &service) != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_name;
    }

    // create publisher builder
    iox2_port_factory_publisher_builder_h publisher_builder =
        iox2_port_factory_pub_sub_publisher_builder(&service, NULL);

    // set unable to deliver handler
    struct callback_context_t ctx;
    ctx.fail_counter = 0;
    iox2_port_factory_publisher_builder_set_unable_to_deliver_handler(
        &publisher_builder, unable_to_deliver_handler, &ctx);

    // create publisher
    iox2_publisher_h publisher = NULL;
    if (iox2_port_factory_publisher_builder_create(publisher_builder, NULL, &publisher) != IOX2_OK) {
        printf("Unable to create publisher!\n");
        goto drop_service;
    }

    int32_t counter = 0;
    const uint64_t DURATION_0S = 0;
    const uint32_t DURATION_500MS = 500000000;
    while (iox2_node_wait(&node_handle, DURATION_0S, DURATION_500MS) == IOX2_OK) {
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
        payload->funky = counter * 812.12;

        // send sample
        printf("Sending sample %d ...\n", counter);
        if (iox2_sample_mut_send(sample, NULL) != IOX2_OK) {
            printf("Failed to send sample\n");
        }
    }


drop_publisher:
    iox2_publisher_drop(publisher);

drop_service:
    iox2_port_factory_pub_sub_drop(service);

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}

// NOLINTEND(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers)
