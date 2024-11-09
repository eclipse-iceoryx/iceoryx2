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

#include <stdint.h>
#include <stdio.h>
#include <string.h>

struct CallbackContext {
    iox2_waitset_guard_h_ref guard_1;
    iox2_waitset_guard_h_ref guard_2;
    iox2_listener_h_ref listener_1;
    iox2_listener_h_ref listener_2;
    const char* service_name_1;
    const char* service_name_2;
};


// the function that is called when a listener has received an event
iox2_callback_progression_e on_event(iox2_waitset_attachment_id_h attachment_id, void* context) {
    struct CallbackContext* ctx = (struct CallbackContext*) context;

    iox2_event_id_t event_id;
    bool has_received_event = false;
    // check if the event originated from guard_1 of listener_1
    if (iox2_waitset_attachment_id_has_event_from(&attachment_id, ctx->guard_1)) {
        printf("Received trigger from \"%s\" ::", ctx->service_name_1);
        do {
            // IMPORTANT:
            // We need to collect all notifications since the WaitSet will wake us up as long as
            // there is something to read. If we skip this step completely we will end up in a
            // busy loop.
            if (iox2_listener_try_wait_one(ctx->listener_1, &event_id, &has_received_event) != IOX2_OK) {
                printf("failed to receive event on listener: %s\n", ctx->service_name_1);
            }

            if (has_received_event) {
                printf(" %lu", (long unsigned) event_id.value);
            }
        } while (has_received_event);
        printf("\n");
        // check if the event originated from guard_2 of listener_2
    } else if (iox2_waitset_attachment_id_has_event_from(&attachment_id, ctx->guard_2)) {
        printf("Received trigger from \"%s\" ::", ctx->service_name_2);
        do {
            if (iox2_listener_try_wait_one(ctx->listener_2, &event_id, &has_received_event) != IOX2_OK) {
                printf("failed to receive event on listener: %s\n", ctx->service_name_2);
            }

            if (has_received_event) {
                printf(" %lu", (long unsigned) event_id.value);
            }
        } while (has_received_event);
        printf("\n");
    }

    iox2_waitset_attachment_id_drop(attachment_id);
    return iox2_callback_progression_e_CONTINUE;
}

//NOLINTBEGIN(readability-function-size)
int main(int argc, char** argv) {
    if (argc != 3) {
        printf("Usage: %s SERVICE_NAME_1 SERVICE_NAME_2\n", argv[0]);
        return -1;
    }

    // create new node
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    iox2_node_h node_handle = NULL;
    if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_IPC, &node_handle) != IOX2_OK) {
        printf("Could not create node!\n");
        goto end;
    }

    // create service names
    iox2_service_name_h service_name_1 = NULL;
    if (iox2_service_name_new(NULL, argv[1], strlen(argv[1]), &service_name_1) != IOX2_OK) {
        printf("Unable to create service name!\n");
        goto drop_node;
    }

    iox2_service_name_h service_name_2 = NULL;
    if (iox2_service_name_new(NULL, argv[2], strlen(argv[2]), &service_name_2) != IOX2_OK) {
        printf("Unable to create service name!\n");
        goto drop_service_name_1;
    }

    // create services
    iox2_service_name_ptr service_name_ptr_1 = iox2_cast_service_name_ptr(service_name_1);
    iox2_service_builder_h service_builder_1 = iox2_node_service_builder(&node_handle, NULL, service_name_ptr_1);
    iox2_service_builder_event_h service_builder_event_1 = iox2_service_builder_event(service_builder_1);
    iox2_port_factory_event_h service_1 = NULL;
    if (iox2_service_builder_event_open_or_create(service_builder_event_1, NULL, &service_1) != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_name_2;
    }

    iox2_service_name_ptr service_name_ptr_2 = iox2_cast_service_name_ptr(service_name_2);
    iox2_service_builder_h service_builder_2 = iox2_node_service_builder(&node_handle, NULL, service_name_ptr_2);
    iox2_service_builder_event_h service_builder_event_2 = iox2_service_builder_event(service_builder_2);
    iox2_port_factory_event_h service_2 = NULL;
    if (iox2_service_builder_event_open_or_create(service_builder_event_2, NULL, &service_2) != IOX2_OK) {
        printf("Unable to create service!\n");
        goto drop_service_1;
    }

    // create listeners
    iox2_port_factory_listener_builder_h listener_builder_1 =
        iox2_port_factory_event_listener_builder(&service_1, NULL);
    iox2_listener_h listener_1 = NULL;
    if (iox2_port_factory_listener_builder_create(listener_builder_1, NULL, &listener_1) != IOX2_OK) {
        printf("Unable to create listener!\n");
        goto drop_service_2;
    }

    iox2_port_factory_listener_builder_h listener_builder_2 =
        iox2_port_factory_event_listener_builder(&service_2, NULL);
    iox2_listener_h listener_2 = NULL;
    if (iox2_port_factory_listener_builder_create(listener_builder_2, NULL, &listener_2) != IOX2_OK) {
        printf("Unable to create listener!\n");
        goto drop_listener_1;
    }

    // create waitset
    iox2_waitset_builder_h waitset_builder = NULL;
    iox2_waitset_builder_new(NULL, &waitset_builder);
    iox2_waitset_h waitset = NULL;
    if (iox2_waitset_builder_create(waitset_builder, iox2_service_type_e_IPC, NULL, &waitset) != IOX2_OK) {
        printf("Unable to create waitset\n");
        goto drop_waitset_builder;
    }

    // attach listeners to waitset
    iox2_waitset_guard_h guard_1 = NULL;
    if (iox2_waitset_attach_notification(&waitset, iox2_listener_get_file_descriptor(&listener_1), NULL, &guard_1)
        != IOX2_OK) {
        printf("Unable to attach listener 1\n");
        goto drop_waitset;
    }

    iox2_waitset_guard_h guard_2 = NULL;
    if (iox2_waitset_attach_notification(&waitset, iox2_listener_get_file_descriptor(&listener_2), NULL, &guard_2)
        != IOX2_OK) {
        printf("Unable to attach listener 2\n");
        goto drop_guard_1;
    }

    struct CallbackContext context;
    context.guard_1 = &guard_1;
    context.guard_2 = &guard_2;
    context.listener_1 = &listener_1;
    context.listener_2 = &listener_2;
    context.service_name_1 = argv[1];
    context.service_name_2 = argv[2];

    iox2_waitset_run_result_e result = iox2_waitset_run_result_e_STOP_REQUEST;
    // loops until the user has pressed CTRL+c, the application has received a SIGTERM or SIGINT
    // signal or the user has called explicitly `iox2_waitset_stop` in the `on_event` function. We
    // didn't add this to the example so feel free to play around with it.
    if (iox2_waitset_wait_and_process(&waitset, on_event, (void*) &context, &result) != IOX2_OK) {
        printf("Failure in WaitSet::wait_and_process loop \n");
    }

    //[unused-label] drop_guard_2:
    iox2_waitset_guard_drop(guard_2);

drop_guard_1:
    iox2_waitset_guard_drop(guard_1);

drop_waitset:
    iox2_waitset_drop(waitset);

drop_waitset_builder:
    iox2_waitset_builder_drop(waitset_builder);

    //[unused-label] drop_listener_2:
    iox2_listener_drop(listener_2);

drop_listener_1:
    iox2_listener_drop(listener_1);

drop_service_2:
    iox2_port_factory_event_drop(service_2);

drop_service_1:
    iox2_port_factory_event_drop(service_1);

drop_service_name_2:
    iox2_service_name_drop(service_name_2);

drop_service_name_1:
    iox2_service_name_drop(service_name_1);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
//NOLINTEND(readability-function-size)
