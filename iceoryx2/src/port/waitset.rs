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

//! A [`WaitSet`](crate::port::waitset::WaitSet) is an implementation of an event multiplexer
//! (Reactor of the reactor design pattern). It allows the user to attach notifications,
//! deadlines or ticks.
//!
//! * **Notification** - An object that emits an event. Whenever the event is detected the
//!     [`WaitSet`](crate::port::waitset::WaitSet) wakes up and informs the user.
//!     Typical use case are gateways, which receives and forwards data whenever new data
//!     is available.
//! * **Deadline** - Like a *Notification* with the exception that the *Deadline* expects an
//!     event after a certain predefined timeout. If the event does not arrive before the
//!     timeout has passed, the [`WaitSet`](crate::port::waitset::WaitSet) wakes up and informs
//!     the user that th *Deadline* has missed its timeout.
//!     Whenever a *Deadline* receives an event, the timeout is reset.
//!     One example is a sensor that shall send an update every 100ms. If after 100ms an update
//!     is not available the application must wake up and take counter measures. If the update
//!     arrives already after 78ms, the timeout is reset back to 100ms.
//! * **Tick** - A cyclic timeout after which the [`WaitSet`](crate::port::waitset::WaitSet)
//!     wakes up and informs the user that the timeout has passed by providing a tick.
//!     This is useful when a [`Publisher`](crate::port::publisher::Publisher) shall send an
//!     heartbeat every 100ms.
//!
//! The [`WaitSet`](crate::port::waitset::WaitSet) allows the user to attach multiple
//! [`Listener`](crate::port::listener::Listener) from multiple [`Node`](crate::node::Node)s,
//! anything that implements
//! [`SynchronousMultiplexing`](iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing)
//! with timeouts (Deadline) or without them (Notification). Additional, an arbitrary amount of
//! cyclic wakeup timeouts (Ticks) can be attached.
//!
//! # Example
//!
//! ## Notification
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! # use core::time::Duration;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let event = node.service_builder(&"MyEventName_1".try_into()?)
//! #     .event()
//! #     .open_or_create()?;
//!
//! let mut listener = event.listener_builder().create()?;
//!
//! let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
//! let guard = waitset.attach_notification(&listener)?;
//!
//! let event_handler = |attachment_id: AttachmentId<ipc::Service>| {
//!     if attachment_id.event_from(&guard) {
//!         while let Ok(Some(event_id)) = listener.try_wait_one() {
//!             println!("received notification {:?}", event_id);
//!         }
//!     }
//! };
//!
//! while waitset.run(event_handler) != Ok(WaitEvent::TerminationRequest) {}
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Deadline
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! # use core::time::Duration;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let event = node.service_builder(&"MyEventName_1".try_into()?)
//! #     .event()
//! #     .open_or_create()?;
//!
//! let mut listener = event.listener_builder().create()?;
//!
//! let listener_deadline = Duration::from_secs(1);
//! let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
//! let guard = waitset.attach_deadline(&listener, listener_deadline)?;
//!
//! let event_handler = |attachment_id: AttachmentId<ipc::Service>| {
//!     if attachment_id.event_from(&guard) {
//!         while let Ok(Some(event_id)) = listener.try_wait_one() {
//!             println!("received notification {:?}", event_id);
//!         }
//!     } else if attachment_id.deadline_from(&guard) {
//!         println!("Oh no, we hit the deadline without receiving any kind of event");
//!     }
//! };
//!
//! while waitset.run(event_handler) != Ok(WaitEvent::TerminationRequest) {}
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Tick
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! # use core::time::Duration;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let pubsub = node.service_builder(&"MyServiceName".try_into()?)
//! #     .publish_subscribe::<u64>()
//! #     .open_or_create()?;
//!
//! let publisher_1 = pubsub.publisher_builder().create()?;
//! let publisher_2 = pubsub.publisher_builder().create()?;
//!
//! let pub_1_period = Duration::from_millis(250);
//! let pub_2_period = Duration::from_millis(718);
//!
//! let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
//! let guard_1 = waitset.attach_tick(pub_1_period)?;
//! let guard_2 = waitset.attach_tick(pub_2_period)?;
//!
//! let event_handler = |attachment_id: AttachmentId<ipc::Service>| {
//!     if attachment_id.event_from(&guard_1) {
//!         publisher_1.send_copy(123);
//!     } else if attachment_id.event_from(&guard_2) {
//!         publisher_2.send_copy(456);
//!     }
//! };
//!
//! while waitset.run(event_handler) != Ok(WaitEvent::TerminationRequest) {}
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
//! let listener_1 = event_1.listener_builder().create()?;
//! let listener_2 = event_2.listener_builder().create()?;
//!
//! let mut listeners: HashMap<AttachmentId<ipc::Service>, &Listener<ipc::Service>> = HashMap::new();
//! let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
//!
//! // attach all listeners to the waitset
//! let guard_1 = waitset.attach_notification(&listener_1)?;
//! let guard_2 = waitset.attach_notification(&listener_2)?;
//! listeners.insert(guard_1.to_attachment_id(), &listener_1);
//! listeners.insert(guard_2.to_attachment_id(), &listener_2);
//!
//! while waitset.run(|attachment_id| {
//!     if let Some(listener) = listeners.get(&attachment_id) {
//!         while let Ok(Some(event_id)) = listener.try_wait_one() {
//!             println!("received notification {:?}", event_id);
//!         }
//!     }
//! }) != Ok(WaitEvent::TerminationRequest) {}
//!
//! # Ok(())
//! # }
//! ```
//!

use std::{
    cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData,
    time::Duration,
};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    deadline_queue::{DeadlineQueue, DeadlineQueueBuilder, DeadlineQueueGuard, DeadlineQueueIndex},
    file_descriptor_set::SynchronousMultiplexing,
    signal::SignalHandler,
};
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum AttachmentIdType {
    Tick(u64, DeadlineQueueIndex),
    Deadline(u64, i32, DeadlineQueueIndex),
    Notification(u64, i32),
}

/// Represents an attachment to the [`WaitSet`]
#[derive(Debug, Clone, Copy)]
pub struct AttachmentId<Service: crate::service::Service> {
    attachment_type: AttachmentIdType,
    _data: PhantomData<Service>,
}

impl<Service: crate::service::Service> PartialEq for AttachmentId<Service> {
    fn eq(&self, other: &Self) -> bool {
        self.attachment_type == other.attachment_type
    }
}

impl<Service: crate::service::Service> Eq for AttachmentId<Service> {}

impl<Service: crate::service::Service> Hash for AttachmentId<Service> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.attachment_type.hash(state)
    }
}

impl<Service: crate::service::Service> AttachmentId<Service> {
    fn tick(waitset: &WaitSet<Service>, deadline_queue_idx: DeadlineQueueIndex) -> Self {
        Self {
            attachment_type: AttachmentIdType::Tick(
                waitset as *const WaitSet<Service> as u64,
                deadline_queue_idx,
            ),
            _data: PhantomData,
        }
    }

    fn deadline(
        waitset: &WaitSet<Service>,
        reactor_idx: i32,
        deadline_queue_idx: DeadlineQueueIndex,
    ) -> Self {
        Self {
            attachment_type: AttachmentIdType::Deadline(
                waitset as *const WaitSet<Service> as u64,
                reactor_idx,
                deadline_queue_idx,
            ),
            _data: PhantomData,
        }
    }

    fn notification(waitset: &WaitSet<Service>, reactor_idx: i32) -> Self {
        Self {
            attachment_type: AttachmentIdType::Notification(
                waitset as *const WaitSet<Service> as u64,
                reactor_idx,
            ),
            _data: PhantomData,
        }
    }

    /// Returns true if an event was emitted from the attachment corresponding to [`Guard`].
    pub fn event_from(&self, other: &Guard<Service>) -> bool {
        if let AttachmentIdType::Deadline(..) = self.attachment_type {
            false
        } else {
            self.attachment_type == other.to_attachment_id().attachment_type
        }
    }

    /// Returns true if the deadline for the attachment corresponding to [`Guard`] was missed.
    pub fn deadline_from(&self, other: &Guard<Service>) -> bool {
        if let AttachmentIdType::Deadline(..) = self.attachment_type {
            self.attachment_type == other.to_attachment_id().attachment_type
        } else {
            false
        }
    }
}

enum GuardType<'waitset, 'attachment, Service: crate::service::Service>
where
    Service::Reactor: 'waitset,
{
    Tick(DeadlineQueueGuard<'waitset>),
    Deadline(
        <Service::Reactor as Reactor>::Guard<'waitset, 'attachment>,
        DeadlineQueueGuard<'waitset>,
    ),
    Notification(<Service::Reactor as Reactor>::Guard<'waitset, 'attachment>),
}

/// Is returned when something is attached to the [`WaitSet`]. As soon as it goes out
/// of scope, the attachment is detached.
pub struct Guard<'waitset, 'attachment, Service: crate::service::Service>
where
    Service::Reactor: 'waitset,
{
    waitset: &'waitset WaitSet<Service>,
    guard_type: GuardType<'waitset, 'attachment, Service>,
}

impl<'waitset, 'attachment, Service: crate::service::Service>
    Guard<'waitset, 'attachment, Service>
{
    /// Extracts the [`AttachmentId`] from the guard.
    pub fn to_attachment_id(&self) -> AttachmentId<Service> {
        match &self.guard_type {
            GuardType::Tick(t) => AttachmentId::tick(self.waitset, t.index()),
            GuardType::Deadline(r, t) => AttachmentId::deadline(
                self.waitset,
                unsafe { r.file_descriptor().native_handle() },
                t.index(),
            ),
            GuardType::Notification(r) => AttachmentId::notification(self.waitset, unsafe {
                r.file_descriptor().native_handle()
            }),
        }
    }
}

impl<'waitset, 'attachment, Service: crate::service::Service> Drop
    for Guard<'waitset, 'attachment, Service>
{
    fn drop(&mut self) {
        if let GuardType::Deadline(r, t) = &self.guard_type {
            self.waitset
                .remove_deadline(unsafe { r.file_descriptor().native_handle() }, t.index())
        }
    }
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
        let deadline_queue = fail!(from self, when DeadlineQueueBuilder::new().create(),
                with WaitSetCreateError::InternalError,
                "{msg} since the underlying Timer could not be created.");

        match <Service::Reactor as Reactor>::Builder::new().create() {
            Ok(reactor) => Ok(WaitSet {
                reactor,
                deadline_queue,
                deadline_to_deadline_queue: RefCell::new(HashMap::new()),
                deadline_queue_to_deadline: RefCell::new(HashMap::new()),
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
    deadline_queue: DeadlineQueue,
    deadline_to_deadline_queue: RefCell<HashMap<i32, DeadlineQueueIndex>>,
    deadline_queue_to_deadline: RefCell<HashMap<DeadlineQueueIndex, i32>>,
}

impl<Service: crate::service::Service> WaitSet<Service> {
    fn remove_deadline<'waitset>(
        &'waitset self,
        reactor_idx: i32,
        deadline_queue_idx: DeadlineQueueIndex,
    ) {
        self.deadline_to_deadline_queue
            .borrow_mut()
            .remove(&reactor_idx);
        self.deadline_queue_to_deadline
            .borrow_mut()
            .remove(&deadline_queue_idx);
    }

    fn contains_deadlines(&self) -> bool {
        !self.deadline_to_deadline_queue.borrow().is_empty()
    }

    fn reset_deadline(
        &self,
        reactor_idx: i32,
    ) -> Result<Option<DeadlineQueueIndex>, WaitSetWaitError> {
        let msg = "Unable to reset deadline";
        if let Some(deadline_queue_idx) = self.deadline_to_deadline_queue.borrow().get(&reactor_idx)
        {
            fail!(from self,
                  when self.deadline_queue.reset(*deadline_queue_idx),
                  with WaitSetWaitError::InternalError,
                  "{msg} since the deadline_queue guard could not be reset for the attachment {reactor_idx}. Continuing operations will lead to invalid deadline failures.");
            Ok(Some(*deadline_queue_idx))
        } else {
            Ok(None)
        }
    }

    /// Attaches an object as notification to the [`WaitSet`]. Whenever an event is received on the
    /// object the [`WaitSet`] informs the user in [`WaitSet::run()`] to handle the event.
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    pub fn attach_notification<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
    ) -> Result<Guard<'waitset, 'attachment, Service>, WaitSetAttachmentError> {
        Ok(Guard {
            waitset: self,
            guard_type: GuardType::Notification(self.attach_to_reactor(attachment)?),
        })
    }

    /// Attaches an object as deadline to the [`WaitSet`]. Whenever the event is received or the
    /// deadline is hit, the user is informed in [`WaitSet::run()`].
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    /// Whenever the object emits an event the deadline is reset by the [`WaitSet`].
    pub fn attach_deadline<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
        deadline: Duration,
    ) -> Result<Guard<'waitset, 'attachment, Service>, WaitSetAttachmentError> {
        let reactor_guard = self.attach_to_reactor(attachment)?;
        let deadline_queue_guard = self.attach_to_deadline_queue(deadline)?;

        self.deadline_to_deadline_queue.borrow_mut().insert(
            unsafe { reactor_guard.file_descriptor().native_handle() },
            deadline_queue_guard.index(),
        );

        Ok(Guard {
            waitset: self,
            guard_type: GuardType::Deadline(reactor_guard, deadline_queue_guard),
        })
    }

    /// Attaches a tick event to the [`WaitSet`]. Whenever the timeout is reached the [`WaitSet`]
    /// informs the user in [`WaitSet::run()`].
    pub fn attach_tick<'waitset>(
        &'waitset self,
        timeout: Duration,
    ) -> Result<Guard<'waitset, '_, Service>, WaitSetAttachmentError> {
        Ok(Guard {
            waitset: self,
            guard_type: GuardType::Tick(self.attach_to_deadline_queue(timeout)?),
        })
    }

    /// Tries to wait on the [`WaitSet`]. The provided callback is called for every attachment that
    /// was triggered and the [`AttachmentId`] is provided as an input argument to acquire the
    /// source.
    /// If nothing was triggered the [`WaitSet`] returns immediately.
    pub fn run<F: FnMut(AttachmentId<Service>)>(
        &self,
        mut fn_call: F,
    ) -> Result<WaitEvent, WaitSetWaitError> {
        if SignalHandler::termination_requested() {
            return Ok(WaitEvent::TerminationRequest);
        }

        let msg = "Unable to call WaitSet::run()";
        let next_timeout = fail!(from self,
                                 when self.deadline_queue.duration_until_next_deadline(),
                                 with WaitSetWaitError::InternalError,
                                 "{msg} since the next timeout could not be acquired.");

        let mut fds = vec![];
        match self.reactor.timed_wait(
            // Collect all triggered file descriptors. We need to collect them first, then reset
            // the deadline and then call the callback, otherwise a long callback may destroy the
            // deadline contract.
            |fd| {
                let fd = unsafe { fd.native_handle() };
                fds.push(fd);
            },
            next_timeout,
        ) {
            Ok(0) => {
                if self.contains_deadlines() {}
                self.deadline_queue
                    .missed_deadlines(|deadline_queue_idx| {
                        fn_call(AttachmentId::tick(self, deadline_queue_idx))
                    })
                    .unwrap();

                Ok(WaitEvent::Tick)
            }
            Ok(n) => {
                // we need to reset the deadlines first, otherwise a long fn_call may extend the
                // deadline unintentionally
                if self.contains_deadlines() {
                    let mut fd_and_deadline_queue_idx = Vec::new();
                    fd_and_deadline_queue_idx.reserve(n);

                    for fd in &fds {
                        fd_and_deadline_queue_idx.push((fd, self.reset_deadline(*fd)?));
                    }

                    for (fd, deadline_queue_idx) in fd_and_deadline_queue_idx {
                        if let Some(deadline_queue_idx) = deadline_queue_idx {
                            fn_call(AttachmentId::deadline(self, *fd, deadline_queue_idx));
                        } else {
                            fn_call(AttachmentId::notification(self, *fd));
                        }
                    }
                } else {
                    for fd in fds {
                        fn_call(AttachmentId::notification(self, fd));
                    }
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

    fn attach_to_deadline_queue<'waitset>(
        &'waitset self,
        timeout: Duration,
    ) -> Result<DeadlineQueueGuard<'waitset>, WaitSetAttachmentError> {
        let msg = "Unable to attach timeout to underlying Timer";

        match self.deadline_queue.add_cyclic_deadline(timeout) {
            Ok(guard) => Ok(guard),
            Err(e) => {
                fail!(from self, with WaitSetAttachmentError::InternalError,
                    "{msg} since the timeout could not be attached to the underlying deadline_queue due to ({:?}).", e);
            }
        }
    }
}
