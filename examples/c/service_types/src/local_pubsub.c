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

#if defined(_WIN32) || defined(WIN32) || defined(__WIN32__) || defined(_WIN64)
#include <stdio.h>

int main() {
    printf("This example does not run on windows\n");
    return -1;
}
#else
#include "iox2/iceoryx2.h"
#include <pthread.h>
#include <stdalign.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

struct res { // NOLINT
    iox2_node_h node;
    iox2_node_h thread_node;
    iox2_service_name_h service_name;
    iox2_node_name_h node_name;
    iox2_node_name_h thread_node_name;
    iox2_port_factory_pub_sub_h service;
    iox2_port_factory_pub_sub_h thread_service;
    iox2_publisher_h publisher;
    iox2_subscriber_h subscriber;
    pthread_t background_thread;
    bool background_thread_started;
};

static struct res example; // NOLINT

void init_res(struct res* const value) { // NOLINT
    value->node = NULL;
    value->thread_node = NULL;
    value->thread_node_name = NULL;
    value->service_name = NULL;
    value->service = NULL;
    value->thread_service = NULL;
    value->publisher = NULL;
    value->subscriber = NULL;
    value->node_name = NULL;
    value->background_thread_started = false;
}

void drop_res(struct res* const value) { // NOLINT
    (void) value;
    // thread cleanup
    if (value->background_thread_started) {
        pthread_join(value->background_thread, NULL);
    }

    if (value->subscriber != NULL) {
        iox2_subscriber_drop(value->subscriber);
    }

    if (value->thread_service != NULL) {
        iox2_port_factory_pub_sub_drop(value->thread_service);
    }

    if (value->thread_node_name != NULL) {
        iox2_node_name_drop(value->thread_node_name);
    }

    if (value->thread_node != NULL) {
        iox2_node_drop(value->thread_node);
    }

    // main cleanup
    if (value->publisher != NULL) {
        iox2_publisher_drop(value->publisher);
    }

    if (value->service != NULL) {
        iox2_port_factory_pub_sub_drop(value->service);
    }

    if (value->service_name != NULL) {
        iox2_service_name_drop(value->service_name);
    }

    if (value->node_name != NULL) {
        iox2_node_name_drop(value->node_name);
    }

    if (value->node != NULL) {
        iox2_node_drop(value->node);
    }
}

void* background_thread(void* unused) {
    (void) unused;

    // Another node is created inside this thread to communicate with the main thread
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    // Optionally, a name can be provided to the node which helps identifying them later during
    // debugging or introspection
    const char* node_name_value = "threadnode";
    if (iox2_node_name_new(NULL, node_name_value, strlen(node_name_value), &example.thread_node_name)) {
        printf("unable to create node name!\n");
        return NULL;
    }
    iox2_node_name_ptr node_name_ptr = iox2_cast_node_name_ptr(example.thread_node_name);
    iox2_node_builder_set_name(&node_builder_handle, node_name_ptr);

    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_LOCAL, &example.thread_node)
        != IOX2_OK) {
        printf("Could not create node!\n");
        return NULL;
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
        return NULL;
    }

    // create service
    if (iox2_service_builder_pub_sub_open_or_create(service_builder_pub_sub, NULL, &example.thread_service)
        != IOX2_OK) {
        printf("Unable to create service!\n");
        return NULL;
    }

    // create subscriber
    iox2_port_factory_subscriber_builder_h subscriber_builder =
        iox2_port_factory_pub_sub_subscriber_builder(&example.thread_service, NULL);
    if (iox2_port_factory_subscriber_builder_create(subscriber_builder, NULL, &example.subscriber) != IOX2_OK) {
        printf("Unable to create subscriber!\n");
        return NULL;
    }

    while (iox2_node_wait(&example.thread_node, 1, 0) == IOX2_OK) {
        // receive sample
        iox2_sample_h sample = NULL;
        if (iox2_subscriber_receive(&example.subscriber, NULL, &sample) != IOX2_OK) {
            printf("Failed to receive sample\n");
            return NULL;
        }

        if (sample != NULL) {
            uint64_t* payload = NULL;
            iox2_sample_payload(&sample, (const void**) &payload, NULL);
            printf("[thread] received: %llu\n", (long long unsigned) *payload);
            iox2_sample_drop(sample);
        }
    }


    return NULL;
}

int main(void) {
    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    // create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    // Optionally, a name can be provided to the node which helps identifying them later during
    // debugging or introspection
    const char* node_name_value = "mainnode";
    if (iox2_node_name_new(NULL, node_name_value, strlen(node_name_value), &example.node_name)) {
        printf("unable to create node name!\n");
        goto end;
    }
    iox2_node_name_ptr node_name_ptr = iox2_cast_node_name_ptr(example.node_name);
    iox2_node_builder_set_name(&node_builder_handle, node_name_ptr);

    // When choosing `iox2_service_type_e_LOCAL` the service does not use inter-process mechanisms
    // like shared memory or unix domain sockets but mechanisms like socketpairs and heap.
    //
    // Those services can communicate only within a single process.
    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_LOCAL, &example.node) != IOX2_OK) {
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

    if (pthread_create(&example.background_thread, NULL, background_thread, NULL) != 0) {
        printf("unable to start background thread\n");
        goto end;
    }
    example.background_thread_started = true;


    uint64_t counter = 0;
    while (iox2_node_wait(&example.node, 1, 0) == IOX2_OK) {
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
#endif
