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
#include <stdio.h>

int main() {
    printf("This example does not run on windows\n");
    return -1;
}
#else
#include <pthread.h>
#include <stdalign.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

struct res { // NOLINT
    iox2_node_h node;
    iox2_service_name_h service_name;
    iox2_port_factory_pub_sub_h service;
    iox2_subscriber_h subscriber;
    pthread_t background_thread;
    bool background_thread_started;
};

static struct res example; // NOLINT

void init_res(struct res* const value) { // NOLINT
    value->node = NULL;
    value->service_name = NULL;
    value->service = NULL;
    value->subscriber = NULL;
    value->background_thread_started = false;
}

void drop_res(struct res* const value) { // NOLINT
    (void) value;
    if (value->background_thread_started) {
        pthread_join(value->background_thread, NULL);
    }

    if (value->subscriber != NULL) {
        iox2_subscriber_drop(value->subscriber);
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

void* background_thread(void* unused) {
    (void) unused;

    while (iox2_node_wait(&example.node, 1, 0) == IOX2_OK) {
        // receive sample
        iox2_sample_h sample = NULL;
        if (iox2_subscriber_receive(&example.subscriber, NULL, &sample) != IOX2_OK) {
            printf("Failed to receive sample\n");
            break;
        }

        if (sample != NULL) {
            uint64_t* payload = NULL;
            iox2_sample_payload(&sample, (const void**) &payload, NULL);
            printf("[thread] received: %llu\n", (unsigned long long) *payload);
            iox2_sample_drop(sample);
        }
    }

    return NULL;
}

int main(void) {
    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    init_res(&example);

    // create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    // In contrast to Rust, all service variants in C have threadsafe ports but at the cost of
    // an additional mutex lock/unlock call.
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

    // create subscriber
    iox2_port_factory_subscriber_builder_h subscriber_builder =
        iox2_port_factory_pub_sub_subscriber_builder(&example.service, NULL);
    if (iox2_port_factory_subscriber_builder_create(subscriber_builder, NULL, &example.subscriber) != IOX2_OK) {
        printf("Unable to create subscriber!\n");
        goto end;
    }

    // All ports (like Subscriber, Publisher, Server, Client) are threadsafe
    // by default so they can be shared between threads.
    if (pthread_create(&example.background_thread, NULL, background_thread, NULL) != 0) {
        printf("unable to start background thread\n");
        goto end;
    }
    example.background_thread_started = true;

    while (iox2_node_wait(&example.node, 1, 0) == IOX2_OK) {
        // receive sample
        iox2_sample_h sample = NULL;
        if (iox2_subscriber_receive(&example.subscriber, NULL, &sample) != IOX2_OK) {
            printf("Failed to receive sample\n");
            goto end;
        }

        if (sample != NULL) {
            uint64_t* payload = NULL;
            iox2_sample_payload(&sample, (const void**) &payload, NULL);
            printf("[main] received: %llu\n", (unsigned long long) *payload);
            iox2_sample_drop(sample);
        }
    }

end:
    drop_res(&example);
    return 0;
}
#endif
