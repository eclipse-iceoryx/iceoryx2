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

int main(void) {
    // Setup logging
    iox2_set_log_level_from_env_or(iox2_log_level_e_INFO);

    if (iox2_service_list(iox2_service_type_e_IPC, iox2_config_global_config(), list_callback, NULL) != IOX2_OK) {
        printf("Failed to list all services.");
    }
}
