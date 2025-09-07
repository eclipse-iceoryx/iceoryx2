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

#ifndef IOX2_SERVICE_INTERNAL_HELPER_HPP
#define IOX2_SERVICE_INTERNAL_HELPER_HPP

namespace iox2::internal {
template <typename T>
struct PlacementDefault {
    template <typename S>
    static void placement_default(S& payload) {
        new (&payload.user_header_mut()) T();
    }
};

template <>
struct PlacementDefault<void> {
    template <typename S>
    static void placement_default(S& payload) {
        static_cast<void>(payload);
    }
};

} // namespace iox2::internal

#endif
