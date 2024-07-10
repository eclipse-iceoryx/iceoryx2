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

#ifndef IOX2_SERVICE_EVENT_BUILDER_HPP_
#define IOX2_SERVICE_EVENT_BUILDER_HPP_

#include <cstdint>
#include <iox/builder.hpp>
#include <iox/expected.hpp>

#include "attribute_specifier.hpp"
#include "attribute_verifier.hpp"
#include "iox/assertions_addendum.hpp"
#include "port_factory_event.hpp"
#include "service_type.hpp"

namespace iox2 {
enum class EventOpenError {
    /// The [`Service`] does not exist.
    DoesNotExist,
    /// The process has not enough permissions to open the [`Service`]
    InsufficientPermissions,
    /// Some underlying resources of the [`Service`] do not exist which indicate
    /// a corrupted
    /// [`Service`]state.
    ServiceInCorruptedState,
    /// The [`Service`] has the wrong messaging pattern.
    IncompatibleMessagingPattern,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does
    /// not satisfy.
    IncompatibleAttributes,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    InternalFailure,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The [`Service`] supports less
    /// [`Notifier`](crate::port::notifier::Notifier)s than requested.
    DoesNotSupportRequestedAmountOfNotifiers,
    /// The [`Service`] supports less
    /// [`Listener`](crate::port::listener::Listener)s than requested.
    DoesNotSupportRequestedAmountOfListeners,
    /// The [`Service`] supported [`EventId`] is smaller than the requested max
    /// [`EventId`].
    DoesNotSupportRequestedMaxEventId,
    /// The [`Service`] supports less [`Node`](crate::node::Node)s than
    /// requested.
    DoesNotSupportRequestedAmountOfNodes,
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

enum class EventCreateError {
    /// Some underlying resources of the [`Service`] are either missing,
    /// corrupted or unaccessible.
    ServiceInCorruptedState,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    InternalFailure,
    /// Multiple processes are trying to create the same [`Service`].
    IsBeingCreatedByAnotherInstance,
    /// The [`Service`] already exists.
    AlreadyExists,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The process has insufficient permissions to create the [`Service`].
    InsufficientPermissions,
    /// The system has cleaned up the [`Service`] but there are still endpoints
    /// like
    /// [`Publisher`](crate::port::publisher::Publisher) or
    /// [`Subscriber`](crate::port::subscriber::Subscriber) alive or
    /// [`Sample`](crate::sample::Sample) or
    /// [`SampleMut`](crate::sample_mut::SampleMut) in use.
    OldConnectionsStillActive,
};

enum class EventOpenOrCreateError {};

template <ServiceType S>
class ServiceBuilderEvent {
    IOX_BUILDER_PARAMETER(int64_t, max_nodes, -1)
    IOX_BUILDER_PARAMETER(int64_t, event_id_max_value, -1)
    IOX_BUILDER_PARAMETER(int64_t, max_notifiers, -1)
    IOX_BUILDER_PARAMETER(int64_t, max_listeners, -1)

   public:
    iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError>
    open_or_create() && {
        IOX_TODO();
    }

    iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError>
    open_or_create_with_attributes(
        const AttributeVerifier& required_attributes) && {
        IOX_TODO();
    }

    iox::expected<PortFactoryEvent<S>, EventOpenError> open() && { IOX_TODO(); }
    iox::expected<PortFactoryEvent<S>, EventOpenError> open_with_attributes(
        const AttributeVerifier& required_attributes) && {
        IOX_TODO();
    }

    iox::expected<PortFactoryEvent<S>, EventOpenError> create() && {
        IOX_TODO();
    }
    iox::expected<PortFactoryEvent<S>, EventOpenError> create_with_attributes(
        const AttributeSpecifier& attributes) && {
        IOX_TODO();
    }
};

}  // namespace iox2

#endif
