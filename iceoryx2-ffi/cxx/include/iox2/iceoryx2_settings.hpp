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

#ifndef IOX2_ICEORYX2_SETTINGS_HPP
#define IOX2_ICEORYX2_SETTINGS_HPP

#include <cstdint>

namespace iox2 {
constexpr uint64_t NODE_NAME_LENGHT = 128;
constexpr uint64_t SERVICE_NAME_LENGTH = 256;
constexpr uint64_t SERVICE_ID_LENGTH = 32;
constexpr uint64_t ATTRIBUTE_KEY_LENGTH = 64;
constexpr uint64_t ATTRIBUTE_VALUE_LENGTH = 128;
constexpr uint64_t MAX_ATTRIBUTES_PER_SERVICE = 16;
constexpr uint64_t MAX_VALUES_PER_ATTRIBUTE_KEY = 8;
constexpr uint64_t MAX_TYPENAME_LENGTH = 256;
} // namespace iox2

#endif
