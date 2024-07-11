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

#include "attribute_specifier.hpp"
#include "attribute_verifier.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "port_factory_publish_subscribe.hpp"
#include "service_type.hpp"

#include <cstdint>

namespace iox2 {
enum class PublishSubscribeOpenError : uint8_t {
    /// Service could not be openen since it does not exist
    DoesNotExist,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    InternalFailure,
    /// The [`Service`] has the wrong payload type.
    IncompatibleTypes,
    /// The [`Service`] has the wrong messaging pattern.
    IncompatibleMessagingPattern,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does
    /// not satisfy.
    IncompatibleAttributes,
    /// The [`Service`] has a lower minimum buffer size than requested.
    DoesNotSupportRequestedMinBufferSize,
    /// The [`Service`] has a lower minimum history size than requested.
    DoesNotSupportRequestedMinHistorySize,
    /// The [`Service`] has a lower minimum subscriber borrow size than
    /// requested.
    DoesNotSupportRequestedMinSubscriberBorrowedSamples,
    /// The [`Service`] supports less
    /// [`Publisher`](crate::port::publisher::Publisher)s than requested.
    DoesNotSupportRequestedAmountOfPublishers,
    /// The [`Service`] supports less
    /// [`Subscriber`](crate::port::subscriber::Subscriber)s than requested.
    DoesNotSupportRequestedAmountOfSubscribers,
    /// The [`Service`] supports less [`Node`](crate::node::Node)s than
    /// requested.
    DoesNotSupportRequestedAmountOfNodes,
    /// The [`Service`] required overflow behavior is not compatible.
    IncompatibleOverflowBehavior,
    /// The process has not enough permissions to open the [`Service`]
    InsufficientPermissions,
    /// Some underlying resources of the [`Service`] are either missing,
    /// corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The maximum number of [`Node`](crate::node::Node)s have already opened
    /// the [`Service`].
    ExceedsMaxNumberOfNodes,
    /// The [`Service`] is marked for destruction and currently cleaning up
    /// since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the
    /// [`Service`] should be
    /// recreatable.
    IsMarkedForDestruction,
};

enum class PublishSubscribeCreateError : uint8_t {
    /// Some underlying resources of the [`Service`] are either missing,
    /// corrupted or unaccessible.
    ServiceInCorruptedState,
    /// Invalid [`Service`] configuration provided. The
    /// [`Subscriber`](crate::port::subscriber::Subscriber)s buffer size must be
    /// at least the size
    /// of the history. Otherwise, how could it hold the whole history?
    SubscriberBufferMustBeLargerThanHistorySize,
    /// The [`Service`] already exists.
    AlreadyExists,
    /// The process has insufficient permissions to create the [`Service`].
    InsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    InternalFailure,
    /// Multiple processes are trying to create the same [`Service`].
    IsBeingCreatedByAnotherInstance,
    /// The system has cleaned up the [`Service`] but there are still endpoints
    /// like
    /// [`Publisher`](crate::port::publisher::Publisher) or
    /// [`Subscriber`](crate::port::subscriber::Subscriber) alive or
    /// [`Sample`](crate::sample::Sample) or
    /// [`SampleMut`](crate::sample_mut::SampleMut) in use.
    OldConnectionsStillActive,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
};

enum class PublishSubscribeOpenOrCreateError : uint8_t {
};

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
