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

#ifndef IOX2_EXAMPLES_MESSAGE_DATA_H
#define IOX2_EXAMPLES_MESSAGE_DATA_H

#include <stdint.h>

// payload_type_name is equivalent to the payload type name used on the Rust side
// NOLINTNEXTLINE, C idiomatic way for compile time const variables
#define IOX2_PAYLOAD_TYPE_NAME "TransmissionData"
struct TransmissionData {
    int32_t x;
    int32_t y;
    double funky;
};

// user_header_type_name is equivalent to the user header type name used on the Rust side
// NOLINTNEXTLINE, C idiomatic way for compile time const variables
#define IOX2_USER_HEADER_TYPE_NAME "CustomHeader"
struct CustomHeader {
    int32_t version;
    uint64_t timestamp;
};

#endif
