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

    // try to create a service with an invalid value
    {
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

        // the opening of the service will fail since the 'camera_resolution' attribute is '1920x1080' and not
        // '3840x2160'
        iox2_attribute_verifier_require(&attribute_verifier, "camera_resolution", "3840x2160");

        // create service
        iox2_port_factory_pub_sub_h service = NULL;
        if (iox2_service_builder_pub_sub_open_with_attributes(
                service_builder_pub_sub, &attribute_verifier, NULL, &service)
            != IOX2_OK) {
            printf("camera_resolution: 3840x2160 -> not available\n");
        } else {
            printf("Error! Service creation with attribute 'camera_resolution: 3840x2160' was not supposed to be "
                   "successful!\n");
            iox2_port_factory_pub_sub_drop(service);
        }
        iox2_attribute_verifier_drop(attribute_verifier);
    }


    // try to create a service with an non existing key
    {
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

        // the opening of the service will fail since the key is not defined
        iox2_attribute_verifier_require_key(&attribute_verifier, "camera_type");

        // create service
        iox2_port_factory_pub_sub_h service = NULL;
        if (iox2_service_builder_pub_sub_open_with_attributes(
                service_builder_pub_sub, &attribute_verifier, NULL, &service)
            != IOX2_OK) {
            printf("camera_type -> not available\n");
        } else {
            printf("Error! Service creation with attribute 'camera_type' was not supposed to be successful!\n");
            iox2_port_factory_pub_sub_drop(service);
        }
        iox2_attribute_verifier_drop(attribute_verifier);
    }

drop_service_name:
    iox2_service_name_drop(service_name);

drop_node:
    iox2_node_drop(node_handle);

end:
    return 0;
}
