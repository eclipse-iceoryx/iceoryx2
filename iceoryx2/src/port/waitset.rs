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

//! # Example
//!
//! ## Use [`AttachmentId::originates_from()`](crate::port::waitset::AttachmentId)
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
//! let event_handler = |attachment_id| {
//!     let listener = if attachment_id.originates_from(&listener_1) {
//!         &listener_1
//!     } else {
//!         &listener_2
//!     };
//!
//!     while let Ok(Some(event_id)) = listener.try_wait_one() {
//!         println!("received notification {:?}", event_id);
//!     }
//! };
//!
//! while waitset.timed_wait(event_handler, Duration::from_secs(1))
//!     != Ok(WaitEvent::TerminationRequest) {}
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

use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, rc::Rc, time::Duration};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    file_descriptor_set::SynchronousMultiplexing,
    signal::SignalHandler,
    timer::{Timer, TimerBuilder, TimerGuard, TimerIndex},
};
use iceoryx2_cal::reactor::*;
use internal::EventOrigin;

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
#[derive(Debug, PartialEq, Eq)]
pub enum AttachmentId {
    Deadline(TimerIndex),
    Notification(i32),
}

impl AttachmentId {
    /// Returns true if the attachment originated from `other`
    pub fn originates_from<T: Guardable>(&self, other: &T) -> bool {
        match self {
            Self::Deadline(v) => Some(*v) == other.timer_index(),
            Self::Notification(v) => Some(*v) == other.reactor_index(),
        }
    }
}

mod internal {
    use super::*;

    pub trait EventOrigin {
        fn reactor_index(&self) -> Option<i32>;
        fn timer_index(&self) -> Option<TimerIndex>;
    }
}

/// Defines something from which an event can originate.
pub trait Guardable: internal::EventOrigin {}

/// Is returned when something is attached to the [`WaitSet`]. As soon as it goes out
/// of scope, the attachment is detached.
pub struct Guard<'waitset, 'attachment, Service: crate::service::Service>
where
    Service::Reactor: 'waitset,
{
    waitset: &'waitset WaitSet<Service>,
    reactor_guard: Option<<Service::Reactor as Reactor>::Guard<'waitset, 'attachment>>,
    timer_guard: Option<Rc<TimerGuard<'waitset>>>,
}

impl<'waitset, 'attachment, Service: crate::service::Service> Drop
    for Guard<'waitset, 'attachment, Service>
{
    fn drop(&mut self) {
        if let Some(r) = &self.reactor_guard {
            self.waitset
                .remove_deadline(unsafe { r.file_descriptor().native_handle() })
        }
    }
}

impl<'waitset, 'attachment, Service: crate::service::Service> Hash
    for Guard<'waitset, 'attachment, Service>
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.reactor_index().hash(state);
        self.timer_index().hash(state);
    }
}

impl<'waitset, 'attachment, Service: crate::service::Service> internal::EventOrigin
    for Guard<'waitset, 'attachment, Service>
{
    fn timer_index(&self) -> Option<TimerIndex> {
        if let Some(t) = &self.timer_guard {
            Some(t.index())
        } else {
            None
        }
    }

    fn reactor_index(&self) -> Option<i32> {
        if let Some(r) = &self.reactor_guard {
            Some(unsafe { r.file_descriptor().native_handle() })
        } else {
            None
        }
    }
}

impl<'waitset, 'attachment, Service: crate::service::Service> Guardable
    for Guard<'waitset, 'attachment, Service>
{
}

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
    pub fn create<'waitset, Service: crate::service::Service>(
        self,
    ) -> Result<WaitSet<Service>, WaitSetCreateError> {
        let msg = "Unable to create WaitSet";
        let timer = fail!(from self, when TimerBuilder::new().create(),
                with WaitSetCreateError::InternalError,
                "{msg} since the underlying Timer could not be created.");

        match <Service::Reactor as Reactor>::Builder::new().create() {
            Ok(reactor) => Ok(WaitSet {
                reactor,
                timer,
                deadlines: RefCell::new(HashMap::new()),
            }),
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
    timer: Timer,
    deadlines: RefCell<HashMap<i32, TimerIndex>>,
}

impl<Service: crate::service::Service> WaitSet<Service> {
    fn remove_deadline<'waitset>(&'waitset self, value: i32) {
        self.deadlines.borrow_mut().remove(&value);
    }

    /// Attaches an object as notification to the [`WaitSet`]. Whenever an event is received on the
    /// object the [`WaitSet`] informs the user in [`WaitSet::run()`] to handle the event.
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    pub fn notification<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
    ) -> Result<Guard<'waitset, 'attachment, Service>, WaitSetAttachmentError> {
        Ok(Guard {
            waitset: self,
            reactor_guard: Some(self.attach_to_reactor(attachment)?),
            timer_guard: None,
        })
    }

    /// Attaches an object as deadline to the [`WaitSet`]. Whenever the event is received or the
    /// deadline is hit, the user is informed in [`WaitSet::run()`].
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    /// Whenever the object emits an event the deadline is reset by the [`WaitSet`].
    pub fn deadline<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
        deadline: Duration,
    ) -> Result<Guard<'waitset, 'attachment, Service>, WaitSetAttachmentError> {
        let reactor_guard = self.attach_to_reactor(attachment)?;
        let timer_guard = Rc::new(self.attach_to_timer(deadline)?);

        self.deadlines.borrow_mut().insert(
            unsafe { reactor_guard.file_descriptor().native_handle() },
            timer_guard.index(),
        );

        Ok(Guard {
            waitset: self,
            reactor_guard: Some(reactor_guard),
            timer_guard: Some(timer_guard),
        })
    }

    /// Attaches a tick event to the [`WaitSet`]. Whenever the timeout is reached the [`WaitSet`]
    /// informs the user in [`WaitSet::run()`].
    pub fn tick<'waitset>(
        &'waitset self,
        timeout: Duration,
    ) -> Result<Guard<'waitset, '_, Service>, WaitSetAttachmentError> {
        Ok(Guard {
            waitset: self,
            reactor_guard: None,
            timer_guard: Some(Rc::new(self.attach_to_timer(timeout)?)),
        })
    }

    /// Tries to wait on the [`WaitSet`]. The provided callback is called for every attachment that
    /// was triggered and the [`AttachmentId`] is provided as an input argument to acquire the
    /// source.
    /// If nothing was triggered the [`WaitSet`] returns immediately.
    pub fn run<F: FnMut(AttachmentId)>(
        &self,
        mut fn_call: F,
    ) -> Result<WaitEvent, WaitSetWaitError> {
        if SignalHandler::termination_requested() {
            return Ok(WaitEvent::TerminationRequest);
        }

        let msg = "Unable to call WaitSet::run()";
        let next_timeout = fail!(from self,
                                 when self.timer.duration_until_next_timeout(),
                                 with WaitSetWaitError::InternalError,
                                 "{msg} since the next timeout could not be acquired.");

        let mut fds = vec![];
        match self.reactor.timed_wait(
            |fd| {
                let fd = unsafe { fd.native_handle() };
                fds.push(fd);
            },
            next_timeout,
        ) {
            Ok(0) => {
                self.timer
                    .missed_timeouts(|timer_idx| fn_call(AttachmentId::Deadline(timer_idx)))
                    .unwrap();

                Ok(WaitEvent::Tick)
            }
            Ok(_) => {
                // we need to reset the deadlines first, otherwise a long fn_call may extend the
                // deadline unintentionally
                for fd in &fds {
                    if let Some(timer_idx) = self.deadlines.borrow().get(&fd) {
                        fail!(from self,
                            when self.timer.reset(*timer_idx),
                            with WaitSetWaitError::InternalError,
                            "{msg} since the timer guard could not be reset for the attachment {fd}. Continuing operations will lead to invalid deadline failures.");
                    }
                }

                for fd in fds {
                    fn_call(AttachmentId::Notification(fd));
                }
                Ok(WaitEvent::Notification)
            }
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

    fn attach_to_reactor<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
    ) -> Result<<Service::Reactor as Reactor>::Guard<'waitset, 'attachment>, WaitSetAttachmentError>
    {
        let msg = "Unable to attach object to internal reactor";

        match self.reactor.attach(attachment) {
            Ok(guard) => Ok(guard),
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

    fn attach_to_timer<'waitset>(
        &'waitset self,
        timeout: Duration,
    ) -> Result<TimerGuard<'waitset>, WaitSetAttachmentError> {
        let msg = "Unable to attach timeout to underlying Timer";

        match self.timer.cyclic(timeout) {
            Ok(guard) => Ok(guard),
            Err(e) => {
                fail!(from self, with WaitSetAttachmentError::InternalError,
                    "{msg} since the timeout could not be attached to the underlying timer due to ({:?}).", e);
            }
        }
    }
}
