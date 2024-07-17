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

#ifndef IOX2_SERVICE_DETAILS_HPP
#define IOX2_SERVICE_DETAILS_HPP

#include "service_type.hpp"
#include "static_config.hpp"

namespace iox2 {
template <ServiceType>
struct ServiceDetails {
    StaticConfig static_details;
};
} // namespace iox2

#endif
