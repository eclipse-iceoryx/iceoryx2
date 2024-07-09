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
#ifndef IOX2_DYNAMIC_CONFIG_PUBLISH_SUBSCRIBE_HPP_
#define IOX2_DYNAMIC_CONFIG_PUBLISH_SUBSCRIBE_HPP_

#include <cstdint>

namespace iox2 {
class DynamicConfigPublishSubscribe {
   public:
    uint64_t number_of_publishers() const {}
    uint64_t number_of_subscribers() const {}
};
}  // namespace iox2

#endif
