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

#ifndef IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_HPP
#define IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/port_factory_publish_subscribe.hpp"
#include "iox2/service_builder_publish_subscribe_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
template <typename Payload, typename UserHeader, ServiceType S>
class ServiceBuilderPublishSubscribe {
    IOX_BUILDER_OPTIONAL(uint64_t, payload_alignment);
    IOX_BUILDER_OPTIONAL(bool, enable_safe_overflow);
    IOX_BUILDER_OPTIONAL(uint64_t, subscriber_max_borrowed_samples);
    IOX_BUILDER_OPTIONAL(uint64_t, history_size);
    IOX_BUILDER_OPTIONAL(uint64_t, subscriber_max_buffer_size);
    IOX_BUILDER_OPTIONAL(uint64_t, max_subscribers);
    IOX_BUILDER_OPTIONAL(uint64_t, max_publishers);
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);

  public:
    template <typename NewHeader>
    auto user_header() -> ServiceBuilderPublishSubscribe<Payload, NewHeader, S> {
        IOX_TODO();
    }

    auto open_or_create() && -> iox::expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                              PublishSubscribeOpenOrCreateError> {
        IOX_TODO();
    }

    auto open_or_create_with_attributes(
        const AttributeVerifier&
            required_attributes) && -> iox::expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                                     PublishSubscribeOpenOrCreateError> {
        IOX_TODO();
    }

    auto open() && -> iox::expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeOpenError> {
        IOX_TODO();
    }
    auto open_with_attributes(
        const AttributeVerifier&
            required_attributes) && -> iox::expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                                     PublishSubscribeOpenError> {
        IOX_TODO();
    }

    auto create() && -> iox::expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>, PublishSubscribeOpenError> {
        IOX_TODO();
    }
    auto create_with_attributes(
        const AttributeSpecifier& attributes) && -> iox::expected<PortFactoryPublishSubscribe<S, Payload, UserHeader>,
                                                                  PublishSubscribeOpenError> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
