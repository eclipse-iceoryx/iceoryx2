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

//! # Example
//!
//! ## Use [`AttachmentId::originates_from()`]
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! # use core::time::Duration;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let event_1 = node.service_builder(&"MyEventName_1".try_into()?)
//! #     .event()
//! #     .open_or_create()?;
//! # let event_2 = node.service_builder(&"MyEventName_2".try_into()?)
//! #     .event()
//! #     .open_or_create()?;
//!
//! let mut listener_1 = event_1.listener_builder().create()?;
//! let mut listener_2 = event_2.listener_builder().create()?;
//!
//! let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
//! let _guard_1 = waitset.attach(&listener_1)?;
//! let _guard_2 = waitset.attach(&listener_2)?;
//!
//! while waitset.timed_wait(|attachment_id| {
//!     let listener = if attachment_id.originates_from(&listener_1) {
//!         &listener_1
//!     } else {
//!         &listener_2
//!     };
//!
//!     while let Ok(Some(event_id)) = listener.try_wait_one() {
//!         println!("received notification {:?}", event_id);
//!     }
//! }, Duration::from_secs(1)) != Ok(WaitEvent::TerminationRequest) {}
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## [`HashMap`](std::collections::HashMap) approach
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! use std::collections::HashMap;
//! use iceoryx2::port::listener::Listener;
//! # use core::time::Duration;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let event_1 = node.service_builder(&"MyEventName_1".try_into()?)
//! #     .event()
//! #     .open_or_create()?;
//! # let event_2 = node.service_builder(&"MyEventName_2".try_into()?)
//! #     .event()
//! #     .open_or_create()?;
//!
//! let mut listeners: HashMap<AttachmentId, Listener<ipc::Service>> = HashMap::new();
//! let listener = event_1.listener_builder().create()?;
//! listeners.insert(AttachmentId::new(&listener), listener);
//!
//! let listener = event_2.listener_builder().create()?;
//! listeners.insert(AttachmentId::new(&listener), listener);
//!
//! let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
//! let mut guards = vec![];
//! for listener in listeners.values() {
//!     guards.push(waitset.attach(listener)?);
//! }
//!
//! while waitset.timed_wait(|attachment_id| {
//!     if let Some(listener) = listeners.get(&attachment_id) {
//!         while let Ok(Some(event_id)) = listener.try_wait_one() {
//!             println!("received notification {:?}", event_id);
//!         }
//!     }
//! }, Duration::from_secs(1)) != Ok(WaitEvent::TerminationRequest) {}
//!
//! # Ok(())
//! # }
//! ```
//!

use std::{fmt::Debug, time::Duration};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{file_descriptor_set::SynchronousMultiplexing, signal::SignalHandler};
use iceoryx2_cal::reactor::*;

/// Defines the type of that triggered [`WaitSet::try_wait()`], [`WaitSet::timed_wait()`] or
/// [`WaitSet::blocking_wait()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WaitEvent {
    /// A termination signal `SIGTERM` was received.
    TerminationRequest,
    /// An interrupt signal `SIGINT` was received.
    Interrupt,
    /// No event was triggered.
    Tick,
    /// One or more event notifications were received.
    Notification,
}

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

impl Default for WaitSetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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

    fn wait<F: FnMut() -> Result<usize, ReactorWaitError>>(
        &self,
        msg: &str,
        mut wait_call: F,
    ) -> Result<WaitEvent, WaitSetWaitError> {
        if SignalHandler::termination_requested() {
            return Ok(WaitEvent::TerminationRequest);
        }

        let result = wait_call();

        match result {
            Ok(0) => Ok(WaitEvent::Tick),
            Ok(_) => Ok(WaitEvent::Notification),
            Err(ReactorWaitError::Interrupt) => Ok(WaitEvent::Interrupt),
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
    pub fn try_wait<F: FnMut(AttachmentId)>(
        &self,
        mut fn_call: F,
    ) -> Result<WaitEvent, WaitSetWaitError> {
        self.wait("Unable to try_wait", || {
            self.reactor
                .try_wait(|fd| fn_call(AttachmentId(unsafe { fd.native_handle() })))
        })
    }

    /// Waits with a timeout on the [`WaitSet`]. The provided callback is called for every
    /// attachment that was triggered and the [`AttachmentId`] is provided as an input argument to
    /// acquire the source.
    /// If nothing was triggered the [`WaitSet`] returns after the timeout has passed.
    pub fn timed_wait<F: FnMut(AttachmentId)>(
        &self,
        mut fn_call: F,
        timeout: Duration,
    ) -> Result<WaitEvent, WaitSetWaitError> {
        self.wait("Unable to timed_wait", || {
            self.reactor.timed_wait(
                |fd| fn_call(AttachmentId(unsafe { fd.native_handle() })),
                timeout,
            )
        })
    }

    /// Blocks on the [`WaitSet`] until at least one event was triggered. The provided callback is
    /// called for every attachment that was triggered and the [`AttachmentId`] is provided as an
    /// input argument to acquire the source.
    /// If nothing was triggered the [`WaitSet`] returns only when the process receives an
    /// interrupt signal.
    pub fn blocking_wait<F: FnMut(AttachmentId)>(
        &self,
        mut fn_call: F,
    ) -> Result<WaitEvent, WaitSetWaitError> {
        self.wait("Unable to blocking_wait", || {
            self.reactor
                .blocking_wait(|fd| fn_call(AttachmentId(unsafe { fd.native_handle() })))
        })
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
