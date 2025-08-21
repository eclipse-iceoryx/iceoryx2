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

#ifndef IOX2_SERVICE_BUILDER_BLACKBOARD_ERROR_HPP
#define IOX2_SERVICE_BUILDER_BLACKBOARD_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Errors that can occur when an existing [`MessagingPattern::Blackboard`] [`Service`] shall be opened.
enum class BlackboardOpenError : uint8_t {
    /// The [`Service`] could not be opened since it does not exist.
    DoesNotExist,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`] has the wrong key type.
    IncompatibleKeys,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The [`AttributeVerifier`] required attributes that the [`Service`] does not satisfy.
    IncompatibleAttributes,
    /// The [`Service`] has the wrong messaging pattern.
    IncompatibleMessagingPattern,
    /// The [`Service`] supports less [`Reader`](crate::port::reader::Reader)s than requested.
    DoesNotSupportRequestedAmountOfReaders,
    /// The process has not enough permissions to open the [`Service`]
    InsufficientPermissions,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// The [`Service`] is marked for destruction and currently cleaning up since no one is using it anymore.
    /// When the call creation call is repeated with a little delay the [`Service`] should be
    /// recreatable.
    IsMarkedForDestruction,
    /// The maximum number of [`Node`](crate::node::Node)s have already opened the [`Service`].
    ExceedsMaxNumberOfNodes,
    /// The [`Service`] supports less [`Node`](crate::node::Node)s than requested.
    DoesNotSupportRequestedAmountOfNodes,
};

/// Errors that can occur when a new [`MessagingPattern::Blackboard`] [`Service`] shall be created.
enum class BlackboardCreateError : uint8_t {
    /// The [`Service`] already exists.
    AlreadyExists,
    /// Multiple processes are trying to create the same [`Service`].
    IsBeingCreatedByAnotherInstance,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
    /// The process has insufficient permissions to create the [`Service`].
    InsufficientPermissions,
    /// Some underlying resources of the [`Service`] are either missing, corrupted or unaccessible.
    ServiceInCorruptedState,
    /// The [`Service`]s creation timeout has passed and it is still not initialized. Can be caused
    /// by a process that crashed during [`Service`] creation.
    HangsInCreation,
    /// No key-value pairs have been provided. At least one is required.
    NoEntriesProvided,
};
} // namespace iox2

#endif
