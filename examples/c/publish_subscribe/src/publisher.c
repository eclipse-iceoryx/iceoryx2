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

int main(void) {
    iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
    iox2_node_h node_handle = NULL;
    int ret_val = iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_IPC, &node_handle);
    if (ret_val != IOX2_OK) {
        printf("Could not create node! Error code: %i", ret_val);
        return -1;
    }

    const uint32_t NUMBER_OF_SECONDS_TO_RUN = 10;
    ret_val = run_publisher(NUMBER_OF_SECONDS_TO_RUN);

    iox2_node_drop(node_handle);

    return ret_val;
}
