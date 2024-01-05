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

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_cal::zero_copy_connection::ZeroCopyCreationError;

/// Defines a failure that can occur when a [`Publisher`] is created with
/// [`crate::service::port_factory::publisher::PortFactoryPublisher`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PublisherCreateError {
    ExceedsMaxSupportedPublishers,
    UnableToCreateDataSegment,
}

impl std::fmt::Display for PublisherCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherCreateError {}

/// Defines a failure that can occur in [`Publisher::loan()`] and [`Publisher::loan_uninit()`] or is part of [`SendCopyError`]
/// emitted in [`Publisher::send_copy()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PublisherLoanError {
    OutOfMemory,
    ExceedsMaxLoanedChunks,
    InternalFailure,
}

impl std::fmt::Display for PublisherLoanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherLoanError {}

enum_gen! {
    /// Failure that can be emitted when a [`crate::sample::Sample`] is sent via [`Publisher::send()`].
    PublisherSendError
  entry:
    SampleDoesNotBelongToPublisher
  mapping:
    PublisherLoanError to LoanError,
    ZeroCopyCreationError to ConnectionError
}

impl std::fmt::Display for PublisherSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherSendError {}

enum_gen! {
    /// Failure that can be emitted when a [`crate::sample::Sample`] is sent via [`Publisher::send_copy()`].
    PublisherSendCopyError
  mapping:
    PublisherLoanError to LoanError,
    ZeroCopyCreationError to ConnectionError
}

impl std::fmt::Display for PublisherSendCopyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for PublisherSendCopyError {}

pub(crate) mod internal {
    use std::fmt::Debug;

    use iceoryx2_cal::zero_copy_connection::PointerOffset;

    pub(crate) trait PublisherMgmt: Debug {
        fn return_loaned_sample(&self, distance_to_chunk: PointerOffset);
        fn publisher_address(&self) -> usize;
    }
}
