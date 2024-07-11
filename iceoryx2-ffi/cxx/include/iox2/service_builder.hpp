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

#ifndef IOX2_SERVICE_BUILDER_HPP_
#define IOX2_SERVICE_BUILDER_HPP_

#include "iox/assertions_addendum.hpp"
#include "service_builder_event.hpp"
#include "service_builder_publish_subscribe.hpp"
#include "service_type.hpp"

namespace iox2 {

template <ServiceType S>
class ServiceBuilder {
  public:
    template <typename Payload>
    ServiceBuilderPublishSubscribe<Payload, void, S> publish_subscribe() {
        IOX_TODO();
    }

    ServiceBuilderEvent<S> event() {
        IOX_TODO();
    }
};

} // namespace iox2
#endif
