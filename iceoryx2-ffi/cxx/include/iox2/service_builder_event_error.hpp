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

#ifndef IOX2_SERVICE_BUILDER_EVENT_ERROR_HPP
#define IOX2_SERVICE_BUILDER_EVENT_ERROR_HPP

#include <cstdint>

namespace iox2 {
enum class EventOpenError : uint8_t {
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

enum class EventCreateError : uint8_t {
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

enum class EventOpenOrCreateError : uint8_t {
    /// The [`Service`] does not exist.
    OpenDoesNotExist,
    /// The process has not enough permissions to open the [`Service`]
    OpenInsufficientPermissions,
    /// Some underlying resources of the [`Service`] do not exist which indicate
    /// a corrupted
    /// [`Service`]state.
    OpenServiceInCorruptedState,
    /// The [`Service`] has the wrong messaging pattern.
    OpenIncompatibleMessagingPattern,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does
    /// not satisfy.
    OpenIncompatibleAttributes,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    OpenInternalFailure,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    OpenHangsInCreation,
    /// The [`Service`] supports less
    /// [`Notifier`](crate::port::notifier::Notifier)s than requested.
    OpenDoesNotSupportRequestedAmountOfNotifiers,
    /// The [`Service`] supports less
    /// [`Listener`](crate::port::listener::Listener)s than requested.
    OpenDoesNotSupportRequestedAmountOfListeners,
    /// The [`Service`] supported [`EventId`] is smaller than the requested max
    /// [`EventId`].
    OpenDoesNotSupportRequestedMaxEventId,
    /// The [`Service`] supports less [`Node`](crate::node::Node)s than
    /// requested.
    OpenDoesNotSupportRequestedAmountOfNodes,
    /// The maximum number of [`Node`](crate::node::Node)s have already opened
    /// the [`Service`].
    OpenExceedsMaxNumberOfNodes,
    /// The [`Service`] is marked for destruction and currently cleaning up
    /// since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the
    /// [`Service`] should be
    /// recreatable.
    OpenIsMarkedForDestruction,

    /// Some underlying resources of the [`Service`] are either missing,
    /// corrupted or unaccessible.
    CreateServiceInCorruptedState,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    CreateInternalFailure,
    /// Multiple processes are trying to create the same [`Service`].
    CreateIsBeingCreatedByAnotherInstance,
    /// The [`Service`] already exists.
    CreateAlreadyExists,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    CreateHangsInCreation,
    /// The process has insufficient permissions to create the [`Service`].
    CreateInsufficientPermissions,
    /// The system has cleaned up the [`Service`] but there are still endpoints
    /// like
    /// [`Publisher`](crate::port::publisher::Publisher) or
    /// [`Subscriber`](crate::port::subscriber::Subscriber) alive or
    /// [`Sample`](crate::sample::Sample) or
    /// [`SampleMut`](crate::sample_mut::SampleMut) in use.
    CreateOldConnectionsStillActive,
};
} // namespace iox2

#endif
