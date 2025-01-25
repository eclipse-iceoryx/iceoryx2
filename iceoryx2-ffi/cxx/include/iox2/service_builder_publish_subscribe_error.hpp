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

#ifndef IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_ERROR_HPP
#define IOX2_SERVICE_BUILDER_PUBLISH_SUBSCRIBE_ERROR_HPP

#include "iox2/iceoryx2.h"
#include <cstdint>

namespace iox2 {
/// Errors that can occur when an existing [`MessagingPattern::PublishSubscribe`] [`Service`] shall be opened.
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
    /// [`Publisher`]s than requested.
    DoesNotSupportRequestedAmountOfPublishers,
    /// The [`Service`] supports less
    /// [`Subscriber`]s than requested.
    DoesNotSupportRequestedAmountOfSubscribers,
    /// The [`Service`] supports less [`Node`]s than
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
    /// The maximum number of [`Node`]s have already opened
    /// the [`Service`].
    ExceedsMaxNumberOfNodes,
    /// The [`Service`] is marked for destruction and currently cleaning up
    /// since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the
    /// [`Service`] should be
    /// recreatable.
    IsMarkedForDestruction,
};

/// Errors that can occur when a new [`MessagingPattern::PublishSubscribe`] [`Service`] shall be created.
enum class PublishSubscribeCreateError : uint8_t {
    /// Some underlying resources of the [`Service`] are either missing,
    /// corrupted or unaccessible.
    ServiceInCorruptedState,
    /// Invalid [`Service`] configuration provided. The
    /// [`Subscriber`]s buffer size must be
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
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
};

/// Errors that can occur when a [`MessagingPattern::PublishSubscribe`] [`Service`] shall be
/// created or opened.
enum class PublishSubscribeOpenOrCreateError : uint8_t {
    /// Service could not be openen since it does not exist
    OpenDoesNotExist,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    OpenInternalFailure,
    /// The [`Service`] has the wrong payload type.
    OpenIncompatibleTypes,
    /// The [`Service`] has the wrong messaging pattern.
    OpenIncompatibleMessagingPattern,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does
    /// not satisfy.
    OpenIncompatibleAttributes,
    /// The [`Service`] has a lower minimum buffer size than requested.
    OpenDoesNotSupportRequestedMinBufferSize,
    /// The [`Service`] has a lower minimum history size than requested.
    OpenDoesNotSupportRequestedMinHistorySize,
    /// The [`Service`] has a lower minimum subscriber borrow size than
    /// requested.
    OpenDoesNotSupportRequestedMinSubscriberBorrowedSamples,
    /// The [`Service`] supports less
    /// [`Publisher`]s than requested.
    OpenDoesNotSupportRequestedAmountOfPublishers,
    /// The [`Service`] supports less
    /// [`Subscriber`]s than requested.
    OpenDoesNotSupportRequestedAmountOfSubscribers,
    /// The [`Service`] supports less [`Node`]s than
    /// requested.
    OpenDoesNotSupportRequestedAmountOfNodes,
    /// The [`Service`] required overflow behavior is not compatible.
    OpenIncompatibleOverflowBehavior,
    /// The process has not enough permissions to open the [`Service`]
    OpenInsufficientPermissions,
    /// Some underlying resources of the [`Service`] are either missing,
    /// corrupted or unaccessible.
    OpenServiceInCorruptedState,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    OpenHangsInCreation,
    /// The maximum number of [`Node`]s have already opened
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
    /// Invalid [`Service`] configuration provided. The
    /// [`Subscriber`]s buffer size must be
    /// at least the size
    /// of the history. Otherwise, how could it hold the whole history?
    CreateSubscriberBufferMustBeLargerThanHistorySize,
    /// The [`Service`] already exists.
    CreateAlreadyExists,
    /// The process has insufficient permissions to create the [`Service`].
    CreateInsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    CreateInternalFailure,
    /// Multiple processes are trying to create the same [`Service`].
    CreateIsBeingCreatedByAnotherInstance,
    /// The system has cleaned up the [`Service`] but there are still endpoints
    /// like
    /// [`Publisher`] or
    /// [`Subscriber`] alive or
    /// [`Sample`] or
    /// [`SampleMut`] in use.
    CreateOldConnectionsStillActive,
    /// The [`Service`]s creation timeout has passed and it is still not
    /// initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    CreateHangsInCreation,
    /// Can occur when another process creates and removes the same [`Service`] repeatedly with a
    /// high frequency.
    SystemInFlux,
};

} // namespace iox2

#endif
