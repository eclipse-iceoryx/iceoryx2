// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use std::{fmt::Debug, time::Duration};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing;
use iceoryx2_cal::reactor::*;

/// Defines the failures that can occur when attaching something with [`WaitSet::attach()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WaitSetAttachmentError {
    /// The [`WaitSet`]s capacity is exceeded.
    InsufficientCapacity,
    /// The attachment is already attached.
    AlreadyAttached,
    /// An internal error has occurred.
    InternalError,
}

impl std::fmt::Display for WaitSetAttachmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "WaitSetAttachmentError::{:?}", self)
    }
}

impl std::error::Error for WaitSetAttachmentError {}

/// Defines the failures that can occur when calling
///  * [`WaitSet::try_wait()`]
///  * [`WaitSet::timed_wait()`]
///  * [`WaitSet::blocking_wait()`]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WaitSetWaitError {
    /// The process received an interrupt signal.
    Interrupt,
    /// The process has not sufficient permissions to wait on the attachments.
    InsufficientPermissions,
    /// An internal error has occurred.
    InternalError,
}

impl std::fmt::Display for WaitSetWaitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "WaitSetWaitError::{:?}", self)
    }
}

impl std::error::Error for WaitSetWaitError {}

/// Defines the failures that can occur when calling [`WaitSetBuilder::create()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WaitSetCreateError {
    /// An internal error has occurred.
    InternalError,
}

impl std::fmt::Display for WaitSetCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "WaitSetCreateError::{:?}", self)
    }
}

impl std::error::Error for WaitSetCreateError {}

/// Represents an attachment to the [`WaitSet`]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct AttachmentId(i32);

impl AttachmentId {
    /// Creates a new [`AttachmentId`] from a [`WaitSet`] attachable object
    pub fn new<T: SynchronousMultiplexing>(other: &T) -> Self {
        Self(unsafe { other.file_descriptor().native_handle() })
    }

    /// Returns true if the attachment originated from [`other`]
    pub fn originates_from<T: SynchronousMultiplexing>(&self, other: &T) -> bool {
        self.0 == Self::new(other).0
    }
}

/// Is returned when something is attached to the [`WaitSet`]. As soon as it goes out
/// of scope, the attachment is detached.
pub struct Guard<'waitset, 'attachment, Service: crate::service::Service>(
    <Service::Reactor as Reactor>::Guard<'waitset, 'attachment>,
)
where
    Service::Reactor: 'waitset;

/// The builder for the [`WaitSet`].
#[derive(Debug)]
pub struct WaitSetBuilder {}

impl WaitSetBuilder {
    /// Creates a new [`WaitSetBuilder`].
    pub fn new() -> Self {
        Self {}
    }

    /// Creates the [`WaitSet`].
    pub fn create<Service: crate::service::Service>(
        self,
    ) -> Result<WaitSet<Service>, WaitSetCreateError> {
        let msg = "Unable to create WaitSet";

        match <Service::Reactor as Reactor>::Builder::new().create() {
            Ok(reactor) => Ok(WaitSet { reactor }),
            Err(ReactorCreateError::UnknownError(e)) => {
                fail!(from self, with WaitSetCreateError::InternalError,
                    "{msg} due to an internal error (error code = {})", e);
            }
        }
    }
}

/// The [`WaitSet`] implements a reactor pattern and allows to wait on multiple events in one
/// single call [`WaitSet::try_wait()`], [`WaitSet::timed_wait()`] or [`WaitSet::blocking_wait()`].
///
/// An struct must implement [`SynchronousMultiplexing`] to be attachable. The
/// [`Listener`](crate::port::listener::Listener) can be attached as well as sockets or anything else that
/// is [`FileDescriptorBased`](iceoryx2_bb_posix::file_descriptor::FileDescriptorBased).
///
/// Can be created via the [`WaitSetBuilder`].
#[derive(Debug)]
pub struct WaitSet<Service: crate::service::Service> {
    reactor: Service::Reactor,
}

impl<Service: crate::service::Service> WaitSet<Service> {
    /// Attaches an object to the [`WaitSet`]. The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    pub fn attach<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
    ) -> Result<Guard<'waitset, 'attachment, Service>, WaitSetAttachmentError> {
        let msg = "Unable to attach the attachment";
        match self.reactor.attach(attachment) {
            Ok(guard) => Ok(Guard(guard)),
            Err(ReactorAttachError::AlreadyAttached) => {
                fail!(from self, with WaitSetAttachmentError::AlreadyAttached,
                    "{msg} {:?} since it is already attached.", attachment);
            }
            Err(ReactorAttachError::CapacityExceeded) => {
                fail!(from self, with WaitSetAttachmentError::AlreadyAttached,
                    "{msg} {:?} since it would exceed the capacity of {} of the waitset.",
                    attachment, self.capacity());
            }
            Err(ReactorAttachError::UnknownError(e)) => {
                fail!(from self, with WaitSetAttachmentError::InternalError,
                    "{msg} {:?} due to an internal error (error code = {})", attachment, e);
            }
        }
    }

    fn wait_result(
        &self,
        msg: &str,
        result: Result<(), ReactorWaitError>,
    ) -> Result<(), WaitSetWaitError> {
        match result {
            Ok(()) => Ok(()),
            Err(ReactorWaitError::Interrupt) => {
                fail!(from self, with WaitSetWaitError::Interrupt,
                    "{msg} since an interrupt signal was received.");
            }
            Err(ReactorWaitError::InsufficientPermissions) => {
                fail!(from self, with WaitSetWaitError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(ReactorWaitError::UnknownError) => {
                fail!(from self, with WaitSetWaitError::InternalError,
                    "{msg} due to an internal error.");
            }
        }
    }

    /// Tries to wait on the [`WaitSet`]. The provided callback is called for every attachment that
    /// was triggered and the [`AttachmentId`] is provided as an input argument to acquire the
    /// source.
    /// If nothing was triggered the [`WaitSet`] returns immediately.
    pub fn try_wait<F: FnMut(AttachmentId)>(&self, mut fn_call: F) -> Result<(), WaitSetWaitError> {
        self.wait_result(
            "Unable to try_wait",
            self.reactor
                .try_wait(|fd| fn_call(AttachmentId(unsafe { fd.native_handle() }))),
        )
    }

    /// Waits with a timeout on the [`WaitSet`]. The provided callback is called for every
    /// attachment that was triggered and the [`AttachmentId`] is provided as an input argument to
    /// acquire the source.
    /// If nothing was triggered the [`WaitSet`] returns after the timeout has passed.
    pub fn timed_wait<F: FnMut(AttachmentId)>(
        &self,
        mut fn_call: F,
        timeout: Duration,
    ) -> Result<(), WaitSetWaitError> {
        self.wait_result(
            "Unable to timed_wait",
            self.reactor.timed_wait(
                |fd| fn_call(AttachmentId(unsafe { fd.native_handle() })),
                timeout,
            ),
        )
    }

    /// Blocks on the [`WaitSet`] until at least one event was triggered. The provided callback is
    /// called for every attachment that was triggered and the [`AttachmentId`] is provided as an
    /// input argument to acquire the source.
    /// If nothing was triggered the [`WaitSet`] returns only when the process receives an
    /// interrupt signal.
    pub fn blocking_wait<F: FnMut(AttachmentId)>(
        &self,
        mut fn_call: F,
    ) -> Result<(), WaitSetWaitError> {
        self.wait_result(
            "Unable to blocking_wait",
            self.reactor
                .blocking_wait(|fd| fn_call(AttachmentId(unsafe { fd.native_handle() }))),
        )
    }

    /// Returns the capacity of the [`WaitSet`]
    pub fn capacity(&self) -> usize {
        self.reactor.capacity()
    }

    /// Returns the number of attachments.
    pub fn len(&self) -> usize {
        self.reactor.len()
    }

    /// Returns true if the [`WaitSet`] has no attachments, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.reactor.is_empty()
    }
}
