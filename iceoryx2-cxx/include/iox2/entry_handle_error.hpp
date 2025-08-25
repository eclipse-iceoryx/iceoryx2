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

#ifndef IOX2_ENTRY_HANDLE_ERROR_HPP
#define IOX2_ENTRY_HANDLE_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Defines a failure that can occur when a [`EntryHandle`] is created with [`Reader::entry()`].
enum class EntryHandleError : uint8_t {
    /// The entry with the given key and value type does not exist.
    EntryDoesNotExist,
};
} // namespace iox2

#endif
