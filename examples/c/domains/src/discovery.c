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
#include <stdio.h>

iox2_callback_progression_e list_callback(const iox2_static_config_t* static_details, void* callback_context) {
    (void) callback_context;
    printf("Found Service: %s, ServiceID: %s\n", static_details->name, static_details->id);
    return iox2_callback_progression_e_CONTINUE;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        printf("usage: %s DOMAIN_NAME\n", argv[0]);
        exit(-1);
    }

    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    // create a new config based on the global config
    iox2_config_ptr config_ptr = iox2_config_global_config();
    iox2_config_h config = NULL;
    iox2_config_from_ptr(config_ptr, NULL, &config);
    config_ptr = iox2_cast_config_ptr(config);

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    if (iox2_config_global_set_prefix(&config, argv[1]) != IOX2_OK) {
        iox2_config_drop(config);
        printf("invalid domain name\"%s\"\n", argv[1]);
        exit(-1);
    }

    printf("\nServices running in domain \"%s\":\n", argv[1]);

    // use the custom config when listing the services
    if (iox2_service_list(iox2_service_type_e_IPC, config_ptr, list_callback, NULL) != IOX2_OK) {
        printf("Failed to list all services.");
    }

    iox2_config_drop(config);
}
