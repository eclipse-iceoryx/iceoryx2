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

#ifndef IOX2_WAITSET_HASH_HPP
#define IOX2_WAITSET_HASH_HPP

#include "iox2/waitset.hpp"

template <>
struct std::hash<iox2::WaitSetAttachmentId<iox2::ServiceType::Ipc>> {
    auto operator()(const iox2::WaitSetAttachmentId<iox2::ServiceType::Ipc>& self) -> std::size_t {
        return self.hash();
    }
};

template <>
struct std::hash<iox2::WaitSetAttachmentId<iox2::ServiceType::Local>> {
    auto operator()(const iox2::WaitSetAttachmentId<iox2::ServiceType::Local>& self) -> std::size_t {
        return self.hash();
    }
};

#endif
