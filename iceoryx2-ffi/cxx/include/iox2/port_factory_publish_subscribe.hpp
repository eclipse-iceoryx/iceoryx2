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

#ifndef IOX2_PORTFACTORY_PUBLISH_SUBSCRIBE_HPP_
#define IOX2_PORTFACTORY_PUBLISH_SUBSCRIBE_HPP_

#include <string>

#include "attribute_set.hpp"
#include "callback_progression.hpp"
#include "dynamic_config_publish_subscribe.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "node_failure_enums.hpp"
#include "node_state.hpp"
#include "port_factory_publisher.hpp"
#include "port_factory_subscriber.hpp"
#include "service_name.hpp"
#include "service_type.hpp"
#include "static_config_publish_subscribe.hpp"

namespace iox2 {
template <ServiceType S, typename Payload, typename UserHeader>
class PortFactoryPublishSubscribe {
  public:
    const ServiceName& service_name() const {
        IOX_TODO();
    }
    const std::string& uuid() const {
        IOX_TODO();
    }
    const AttributeSet& attributes() const {
        IOX_TODO();
    }
    const StaticConfigPublishSubscribe& static_config() const {
        IOX_TODO();
    }
    const DynamicConfigPublishSubscribe& dynamic_config() const {
        IOX_TODO();
    }

    iox::expected<void, NodeListFailure> nodes(const iox::function<CallbackProgression(NodeState<S>)>) const {
        IOX_TODO();
    }

    PortFactorySubscriber<S, Payload, UserHeader> subscriber_builder() const {
        IOX_TODO();
    }
    PortFactoryPublisher<S, Payload, UserHeader> publisher_builder() const {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
