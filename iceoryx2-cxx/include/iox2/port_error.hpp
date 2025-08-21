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

#ifndef IOX2_PORT_ERROR_HPP
#define IOX2_PORT_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Defines a failure that can occur in
/// [`Publisher::loan()`](crate::port::publisher::Publisher::loan()) and
/// [`Publisher::loan_uninit()`](crate::port::publisher::Publisher::loan_uninit())
/// or is part of [`SendError`] emitted in
enum class LoanError : uint8_t {
    /// The data segment does not have any more memory left
    OutOfMemory,
    /// The maximum amount of data a user can borrow is
    /// defined in [`crate::config::Config`]. When this is exceeded those calls will fail.
    ExceedsMaxLoanedSamples,
    /// The provided slice size exceeds the configured max slice size.
    /// To send data with this size a new port has to be created with as a larger slice size or the
    /// port must be configured with an
    /// [`AllocationStrategy`](iceoryx2_cal::shm_allocator::AllocationStrategy).
    ExceedsMaxLoanSize,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
};

/// Failure that can be emitted when data is sent.
enum class SendError : uint8_t {
    /// Send was called but the corresponding port went already out of scope.
    ConnectionBrokenSinceSenderNoLongerExists,
    /// A connection between two ports has been corrupted.
    ConnectionCorrupted,
    /// The data segment does not have any more memory left
    LoanErrorOutOfMemory,
    /// The maximum amount of data a user can borrow is
    /// defined in [`crate::config::Config`]. When this is exceeded those calls will fail.
    LoanErrorExceedsMaxLoans,
    /// The provided slice size exceeds the configured max slice size.
    /// To send data with this size a new port has to be created with as a larger slice size or the
    /// port must be configured with an
    /// [`AllocationStrategy`](iceoryx2_cal::shm_allocator::AllocationStrategy).
    LoanErrorExceedsMaxLoanSize,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    LoanErrorInternalFailure,
    /// A failure occurred while establishing a connection to the ports counterpart port.
    ConnectionError,
};

/// Defines the failure that can occur when receiving data with
/// [`Subscriber::receive()`](crate::port::subscriber::Subscriber::receive()).
enum class ReceiveError : uint8_t {
    /// The maximum amount of data a user can borrow with is
    /// defined in [`crate::config::Config`]. When this is exceeded no more data can be received
    /// until the user has released older data.
    ExceedsMaxBorrows,

    /// Occurs when a receiver is unable to connect to a corresponding sender.
    FailedToEstablishConnection,

    /// Failures when mapping the corresponding data segment
    UnableToMapSendersDataSegment
};

/// Failure that can be emitted when a [`RequestMut`] is sent.
enum class RequestSendError : uint8_t {
    /// Sending this [`RequestMut`] exceeds the maximum supported amount of active
    /// requests. When a [`PendingResponse`] object is released another [`RequestMut`]
    /// can be sent.
    ExceedsMaxActiveRequests,
    /// Send was called but the corresponding port went already out of scope.
    ConnectionBrokenSinceSenderNoLongerExists,
    /// A connection between two ports has been corrupted.
    ConnectionCorrupted,
    /// The data segment does not have any more memory left
    LoanErrorOutOfMemory,
    /// The maximum amount of data a user can borrow is
    /// defined in [`crate::config::Config`]. When this is exceeded those calls will fail.
    LoanErrorExceedsMaxLoans,
    /// The provided slice size exceeds the configured max slice size.
    /// To send data with this size a new port has to be created with as a larger slice size or the
    /// port must be configured with an
    /// [`AllocationStrategy`](iceoryx2_cal::shm_allocator::AllocationStrategy).
    LoanErrorExceedsMaxLoanSize,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    LoanErrorInternalFailure,
    /// A failure occurred while establishing a connection to the ports counterpart port.
    ConnectionError,
};
} // namespace iox2

#endif
