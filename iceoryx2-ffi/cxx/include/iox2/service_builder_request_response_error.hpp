// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#ifndef IOX2_SERVICE_BUILDER_REQUEST_RESPONSE_ERROR_HPP
#define IOX2_SERVICE_BUILDER_REQUEST_RESPONSE_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Errors that can occur when an existing [`MessagingPattern::RequestResponse`] [`Service`] shall
/// be opened.
enum class RequestResponseOpenError : uint8_t {
    /// Service could not be openen since it does not exist
    DoesNotExist,
    /// The [`Service`] has a lower maximum amount of loaned
    /// [`RequestMut`] for a [`Client`].
    DoesNotSupportRequestedAmountOfClientRequestLoans,
    /// The [`Service`] has a lower maximum amount of [`ActiveRequest`]s than
    /// requested.
    DoesNotSupportRequestedAmountOfActiveRequestsPerClient,
    /// The [`Service`] has a lower maximum response buffer size than requested.
    DoesNotSupportRequestedResponseBufferSize,
    /// The [`Service`] has a lower maximum number of servers than requested.
    DoesNotSupportRequestedAmountOfServers,
    /// The [`Service`] has a lower maximum number of clients than requested.
    DoesNotSupportRequestedAmountOfClients,
    /// The [`Service`] has a lower maximum number of nodes than requested.
    DoesNotSupportRequestedAmountOfNodes,
    /// The [`Service`] has a lower maximum number of [`Response`] borrows than requested.
    DoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse,
    /// The maximum number of [`Node`]s have already opened the [`Service`].
    ExceedsMaxNumberOfNodes,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The [`Service`] has the wrong request payload type, request header type or type alignment.
    IncompatibleRequestType,
    /// The [`Service`] has the wrong response payload type, response header type or type alignment.
    IncompatibleResponseType,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does not satisfy.
    IncompatibleAttributes,
    /// The [`Service`] has the wrong messaging pattern.
    IncompatibleMessagingPattern,
    /// The [`Service`] required overflow behavior for requests is not compatible.
    IncompatibleOverflowBehaviorForRequests,
    /// The [`Service`] required overflow behavior for responses is not compatible.
    IncompatibleOverflowBehaviorForResponses,
    /// The [`Service`] does not support the required behavior for fire and forget requests.
    IncompatibleBehaviorForFireAndForgetRequests,
    /// The process has not enough permissions to open the [`Service`].
    InsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The [`Service`] is marked for destruction and currently cleaning up since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the [`Service`] should be
    /// recreatable.
    IsMarkedForDestruction,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
};

/// Errors that can occur when a new [`MessagingPattern::RequestResponse`] [`Service`] shall be created.
enum class RequestResponseCreateError : uint8_t {
    /// The [`Service`] already exists.
    AlreadyExists,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// Multiple processes are trying to create the same [`Service`].
    IsBeingCreatedByAnotherInstance,
    /// The process has insufficient permissions to create the [`Service`].
    InsufficientPermissions,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
};

/// Errors that can occur when a [`MessagingPattern::RequestResponse`] [`Service`] shall be
/// created or opened.
enum class RequestResponseOpenOrCreateError : uint8_t {
    /// Service could not be openen since it does not exist
    OpenDoesNotExist,
    /// The [`Service`] has a lower maximum amount of loaned
    /// [`RequestMut`] for a [`Client`].
    OpenDoesNotSupportRequestedAmountOfClientRequestLoans,
    /// The [`Service`] has a lower maximum amount of [`ActiveRequest`]s than
    /// requested.
    OpenDoesNotSupportRequestedAmountOfActiveRequestsPerClient,
    /// The [`Service`] has a lower maximum response buffer size than requested.
    OpenDoesNotSupportRequestedResponseBufferSize,
    /// The [`Service`] has a lower maximum number of servers than requested.
    OpenDoesNotSupportRequestedAmountOfServers,
    /// The [`Service`] has a lower maximum number of clients than requested.
    OpenDoesNotSupportRequestedAmountOfClients,
    /// The [`Service`] has a lower maximum number of nodes than requested.
    OpenDoesNotSupportRequestedAmountOfNodes,
    /// The [`Service`] has a lower maximum number of [`Response`] borrows than requested.
    OpenDoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse,
    /// The maximum number of [`Node`]s have already opened the [`Service`].
    OpenExceedsMaxNumberOfNodes,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    OpenHangsInCreation,
    /// The [`Service`] has the wrong request payload type, request header type or type alignment.
    OpenIncompatibleRequestType,
    /// The [`Service`] has the wrong response payload type, response header type or type alignment.
    OpenIncompatibleResponseType,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does not satisfy.
    OpenIncompatibleAttributes,
    /// The [`Service`] has the wrong messaging pattern.
    OpenIncompatibleMessagingPattern,
    /// The [`Service`] required overflow behavior for requests is not compatible.
    OpenIncompatibleOverflowBehaviorForRequests,
    /// The [`Service`] required overflow behavior for responses is not compatible.
    OpenIncompatibleOverflowBehaviorForResponses,
    /// The [`Service`] does not support the required behavior for fire and forget requests.
    OpenIncompatibleBehaviorForFireAndForgetRequests,
    /// The process has not enough permissions to open the [`Service`].
    OpenInsufficientPermissions,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    OpenInternalFailure,
    /// The [`Service`] is marked for destruction and currently cleaning up since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the [`Service`] should be
    /// recreatable.
    OpenIsMarkedForDestruction,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    OpenServiceInCorruptedState,

    /// The [`Service`] already exists.
    CreateAlreadyExists,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    CreateInternalFailure,
    /// Multiple processes are trying to create the same [`Service`].
    CreateIsBeingCreatedByAnotherInstance,
    /// The process has insufficient permissions to create the [`Service`].
    CreateInsufficientPermissions,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    CreateHangsInCreation,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    CreateServiceInCorruptedState,

    /// Can occur when another process creates and removes the same [`Service`] repeatedly with a
    /// high frequency.
    SystemInFlux,
};

} // namespace iox2
#endif
