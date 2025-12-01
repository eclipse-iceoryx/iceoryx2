// Copyright 2023, Eclipse Foundation and the iceoryx contributors. All rights reserved.
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

#ifndef IOX2_BB_BUFFER_BUFFER_INFO_HPP
#define IOX2_BB_BUFFER_BUFFER_INFO_HPP

#include <cstdint>

namespace iox2 {
namespace legacy {
/// @brief struct used to define the used size and total size of a buffer
struct BufferInfo {
    uint64_t used_size { 0 };
    uint64_t total_size { 0 };
};

} // namespace legacy
} // namespace iox2

#endif
