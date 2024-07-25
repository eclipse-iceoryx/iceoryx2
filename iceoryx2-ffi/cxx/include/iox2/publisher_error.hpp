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

#ifndef IOX2_PUBLISHER_ERROR_HPP
#define IOX2_PUBLISHER_ERROR_HPP

#include <cstdint>

namespace iox2 {

enum class PublisherCreateError : uint8_t {
    /// The maximum amount of [`Publisher`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more
    /// [`Publisher`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedPublishers,
    /// The datasegment in which the payload of the [`Publisher`] is stored,
    /// could not be created.
    UnableToCreateDataSegment,
};

/// Defines a failure that can occur in [`Publisher::loan()`] and
/// [`Publisher::loan_uninit()`] or is part of [`PublisherSendError`] emitted in
/// [`Publisher::send_copy()`].
enum class PublisherLoanError : uint8_t {
    /// The [`Publisher`]s data segment does not have any more memory left
    OutOfMemory,
    /// The maximum amount of [`SampleMut`]s a user can borrow with
    /// [`Publisher::loan()`] or
    /// [`Publisher::loan_uninit()`] is
    /// defined in [`crate::config::Config`]. When this is exceeded those calls
    /// will fail.
    ExceedsMaxLoanedSamples,
    /// The provided slice size exceeds the configured max slice size of the
    /// [`Publisher`].
    /// To send a [`SampleMut`] with this size a new [`Publisher`] has to be
    /// created with
    /// a
    /// [`crate::service::port_factory::publisher::PortFactoryPublisher::max_slice_len()`]
    /// greater or equal to the required len.
    ExceedsMaxLoanSize,
    /// Errors that indicate either an implementation issue or a wrongly
    /// configured system.
    InternalFailure,
};

/// Failure that can be emitted when a [`SampleMut`] is sent via [`SampleMut::send()`].
enum class PublisherSendError : uint8_t {
    /// [`SampleMut::send()`] was called but the corresponding [`Publisher`] went already out of
    /// scope.
    ConnectionBrokenSincePublisherNoLongerExists,
    /// A connection between a [`Subscriber`] and a
    /// [`Publisher`] is corrupted.
    ConnectionCorrupted,
    /// A failure occurred while acquiring memory for the payload
    /// The [`Publisher`]s data segment does not have any more memory left
    LoanErrorOutOfMemory,
    /// The maximum amount of [`SampleMut`]s a user can borrow with [`Publisher::loan()`] or
    /// [`Publisher::loan_uninit()`] is
    /// defined in [`crate::config::Config`]. When this is exceeded those calls will fail.
    LoanErrorExceedsMaxLoanedSamples,
    /// The provided slice size exceeds the configured max slice size of the [`Publisher`].
    /// To send a [`SampleMut`] with this size a new [`Publisher`] has to be created with
    /// a [`crate::service::port_factory::publisher::PortFactoryPublisher::max_slice_len()`]
    /// greater or equal to the required len.
    LoanErrorExceedsMaxLoanSize,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    LoanErrorInternalFailure,
    /// A failure occurred while establishing a connection to a
    /// [`Subscriber`]
    ConnectionError,
};
} // namespace iox2

#endif
