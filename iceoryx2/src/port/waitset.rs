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
//! deadlines or intervals.
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
//!     One example is a sensor that shall send an update every 100ms and the applications requires
//!     the sensor data latest after 120ms. If after 120ms an update
//!     is not available the application must wake up and take counter measures. If the update
//!     arrives within the timeout, the timeout is reset back to 120ms.
//! * **Interval** - An time period after which the [`WaitSet`](crate::port::waitset::WaitSet)
//!     wakes up and informs the user that the time has passed by.
//!     This is useful when a [`Publisher`](crate::port::publisher::Publisher) shall send an
//!     heartbeat every 100ms.
//!
//! The [`WaitSet`](crate::port::waitset::WaitSet) allows the user to attach multiple
//! [`Listener`](crate::port::listener::Listener) from multiple [`Node`](crate::node::Node)s,
//! anything that implements
//! [`SynchronousMultiplexing`](iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing)
//! with timeouts (Deadline) or without them (Notification). Additional, an arbitrary amount of
//! intervals (Ticks) can be attached.
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
//! let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
//!     if attachment_id.has_event_from(&guard) {
//!         while let Ok(Some(event_id)) = listener.try_wait_one() {
//!             println!("received notification {:?}", event_id);
//!         }
//!     }
//!     CallbackProgression::Continue
//! };
//!
//! waitset.wait_and_process(on_event)?;
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
//! let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
//!     if attachment_id.has_event_from(&guard) {
//!         while let Ok(Some(event_id)) = listener.try_wait_one() {
//!             println!("received notification {:?}", event_id);
//!         }
//!     } else if attachment_id.has_missed_deadline(&guard) {
//!         println!("Oh no, we hit the deadline without receiving any kind of event");
//!     }
//!     CallbackProgression::Continue
//! };
//!
//! waitset.wait_and_process(on_event)?;
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
//! let guard_1 = waitset.attach_interval(pub_1_period)?;
//! let guard_2 = waitset.attach_interval(pub_2_period)?;
//!
//! let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
//!     if attachment_id.has_event_from(&guard_1) {
//!         publisher_1.send_copy(123);
//!     } else if attachment_id.has_event_from(&guard_2) {
//!         publisher_2.send_copy(456);
//!     }
//!     CallbackProgression::Continue
//! };
//!
//! waitset.wait_and_process(on_event)?;
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
//! let mut listeners: HashMap<WaitSetAttachmentId<ipc::Service>, &Listener<ipc::Service>> = HashMap::new();
//! let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
//!
//! // attach all listeners to the waitset
//! let guard_1 = waitset.attach_notification(&listener_1)?;
//! let guard_2 = waitset.attach_notification(&listener_2)?;
//! listeners.insert(WaitSetAttachmentId::from_guard(&guard_1), &listener_1);
//! listeners.insert(WaitSetAttachmentId::from_guard(&guard_2), &listener_2);
//!
//! let on_event = |attachment_id| {
//!     if let Some(listener) = listeners.get(&attachment_id) {
//!         while let Ok(Some(event_id)) = listener.try_wait_one() {
//!             println!("received notification {:?}", event_id);
//!         }
//!     }
//!     CallbackProgression::Continue
//! };
//!
//! waitset.wait_and_process(on_event)?;
//!
//! # Ok(())
//! # }
//! ```
//!

use std::{
    cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData,
    sync::atomic::Ordering, time::Duration,
};

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    deadline_queue::{DeadlineQueue, DeadlineQueueBuilder, DeadlineQueueGuard, DeadlineQueueIndex},
    file_descriptor::FileDescriptor,
    file_descriptor_set::SynchronousMultiplexing,
    signal::SignalHandler,
};
use iceoryx2_cal::reactor::*;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

/// States why the [`WaitSet::wait_and_process()`] method returned.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WaitSetRunResult {
    /// A termination signal `SIGTERM` was received.
    TerminationRequest,
    /// An interrupt signal `SIGINT` was received.
    Interrupt,
    /// The users callback returned [`CallbackProgression::Stop`].
    StopRequest,
    /// All events were handled.
    AllEventsHandled,
}

/// Defines the failures that can occur when attaching something with
/// [`WaitSet::attach_notification()`], [`WaitSet::attach_interval()`] or [`WaitSet::attach_deadline()`].
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

/// Defines the failures that can occur when calling [`WaitSet::wait_and_process()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WaitSetRunError {
    /// The process has not sufficient permissions to wait on the attachments.
    InsufficientPermissions,
    /// An internal error has occurred.
    InternalError,
    /// Waiting on an empty [`WaitSet`] would lead to a deadlock therefore it causes an error.
    NoAttachments,
}

impl std::fmt::Display for WaitSetRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "WaitSetRunError::{:?}", self)
    }
}

impl std::error::Error for WaitSetRunError {}

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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
enum AttachmentIdType {
    Tick(u64, DeadlineQueueIndex),
    Deadline(u64, i32, DeadlineQueueIndex),
    Notification(u64, i32),
}

/// Represents an attachment to the [`WaitSet`]
#[derive(Clone, Copy)]
pub struct WaitSetAttachmentId<Service: crate::service::Service> {
    attachment_type: AttachmentIdType,
    _data: PhantomData<Service>,
}

impl<Service: crate::service::Service> Debug for WaitSetAttachmentId<Service> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WaitSetAttachmentId<{}> {{ attachment_type: {:?} }}",
            core::any::type_name::<Service>(),
            self.attachment_type
        )
    }
}

impl<Service: crate::service::Service> WaitSetAttachmentId<Service> {
    /// Creates an [`WaitSetAttachmentId`] from a [`WaitSetGuard`] that was returned via
    /// [`WaitSet::attach_interval()`], [`WaitSet::attach_notification()`] or
    /// [`WaitSet::attach_deadline()`].
    pub fn from_guard(guard: &WaitSetGuard<Service>) -> Self {
        match &guard.guard_type {
            GuardType::Tick(t) => WaitSetAttachmentId::tick(guard.waitset, t.index()),
            GuardType::Deadline(r, t) => WaitSetAttachmentId::deadline(
                guard.waitset,
                unsafe { r.file_descriptor().native_handle() },
                t.index(),
            ),
            GuardType::Notification(r) => {
                WaitSetAttachmentId::notification(guard.waitset, unsafe {
                    r.file_descriptor().native_handle()
                })
            }
        }
    }
}

impl<Service: crate::service::Service> PartialOrd for WaitSetAttachmentId<Service> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Service: crate::service::Service> Ord for WaitSetAttachmentId<Service> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.attachment_type.cmp(&other.attachment_type)
    }
}

impl<Service: crate::service::Service> PartialEq for WaitSetAttachmentId<Service> {
    fn eq(&self, other: &Self) -> bool {
        self.attachment_type == other.attachment_type
    }
}

impl<Service: crate::service::Service> Eq for WaitSetAttachmentId<Service> {}

impl<Service: crate::service::Service> Hash for WaitSetAttachmentId<Service> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.attachment_type.hash(state)
    }
}

impl<Service: crate::service::Service> WaitSetAttachmentId<Service> {
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

    /// Returns true if an event was emitted from a notification or deadline attachment
    /// corresponding to [`WaitSetGuard`].
    pub fn has_event_from(&self, other: &WaitSetGuard<Service>) -> bool {
        let other_attachment = WaitSetAttachmentId::from_guard(other);
        if let AttachmentIdType::Deadline(other_waitset, other_reactor_idx, _) =
            other_attachment.attachment_type
        {
            if let AttachmentIdType::Notification(waitset, reactor_idx) = self.attachment_type {
                waitset == other_waitset && reactor_idx == other_reactor_idx
            } else {
                false
            }
        } else {
            self.attachment_type == other_attachment.attachment_type
        }
    }

    /// Returns true if the deadline for the attachment corresponding to [`WaitSetGuard`] was missed.
    pub fn has_missed_deadline(&self, other: &WaitSetGuard<Service>) -> bool {
        if let AttachmentIdType::Deadline(..) = self.attachment_type {
            self.attachment_type == WaitSetAttachmentId::from_guard(other).attachment_type
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
pub struct WaitSetGuard<'waitset, 'attachment, Service: crate::service::Service>
where
    Service::Reactor: 'waitset,
{
    waitset: &'waitset WaitSet<Service>,
    guard_type: GuardType<'waitset, 'attachment, Service>,
}

impl<'waitset, 'attachment, Service: crate::service::Service> Drop
    for WaitSetGuard<'waitset, 'attachment, Service>
{
    fn drop(&mut self) {
        if let GuardType::Deadline(r, t) = &self.guard_type {
            self.waitset
                .remove_deadline(unsafe { r.file_descriptor().native_handle() }, t.index())
        }
        self.waitset.detach();
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
    pub fn create<Service: crate::service::Service>(
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
                attachment_to_deadline: RefCell::new(HashMap::new()),
                deadline_to_attachment: RefCell::new(HashMap::new()),
                attachment_counter: IoxAtomicUsize::new(0),
            }),
            Err(ReactorCreateError::UnknownError(e)) => {
                fail!(from self, with WaitSetCreateError::InternalError,
                    "{msg} due to an internal error (error code = {})", e);
            }
        }
    }
}

/// The [`WaitSet`] implements a reactor pattern and allows to wait on multiple events in one
/// single call [`WaitSet::wait_and_process_once()`] until it wakes up or to run repeatedly with
/// [`WaitSet::wait_and_process()`] until the a interrupt or termination signal was received or the user
/// has explicitly requested to stop by returning [`CallbackProgression::Stop`] in the provided
/// callback.
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
    attachment_to_deadline: RefCell<HashMap<i32, DeadlineQueueIndex>>,
    deadline_to_attachment: RefCell<HashMap<DeadlineQueueIndex, i32>>,
    attachment_counter: IoxAtomicUsize,
}

impl<Service: crate::service::Service> WaitSet<Service> {
    fn detach(&self) {
        self.attachment_counter.fetch_sub(1, Ordering::Relaxed);
    }

    fn attach(&self) -> Result<(), WaitSetAttachmentError> {
        if self.len() == self.capacity() {
            fail!(from self, with WaitSetAttachmentError::InsufficientCapacity,
                    "Unable to add attachment since it would exceed the capacity of {}.", self.capacity());
        }

        self.attachment_counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn remove_deadline(&self, reactor_idx: i32, deadline_queue_idx: DeadlineQueueIndex) {
        self.attachment_to_deadline
            .borrow_mut()
            .remove(&reactor_idx);
        self.deadline_to_attachment
            .borrow_mut()
            .remove(&deadline_queue_idx);
    }

    fn reset_deadline(
        &self,
        reactor_idx: i32,
    ) -> Result<Option<DeadlineQueueIndex>, WaitSetRunError> {
        let msg = "Unable to reset deadline";
        if let Some(deadline_queue_idx) = self.attachment_to_deadline.borrow().get(&reactor_idx) {
            fail!(from self,
                  when self.deadline_queue.reset(*deadline_queue_idx),
                  with WaitSetRunError::InternalError,
                  "{msg} since the deadline_queue guard could not be reset for the attachment {reactor_idx}. Continuing operations will lead to invalid deadline failures.");
            Ok(Some(*deadline_queue_idx))
        } else {
            Ok(None)
        }
    }

    fn handle_deadlines<F: FnMut(WaitSetAttachmentId<Service>) -> CallbackProgression>(
        &self,
        fn_call: &mut F,
        error_msg: &str,
    ) -> Result<WaitSetRunResult, WaitSetRunError> {
        let deadline_to_attachment = self.deadline_to_attachment.borrow();
        let mut result = WaitSetRunResult::AllEventsHandled;
        let call = |idx: DeadlineQueueIndex| -> CallbackProgression {
            let progression = if let Some(reactor_idx) = deadline_to_attachment.get(&idx) {
                fn_call(WaitSetAttachmentId::deadline(self, *reactor_idx, idx))
            } else {
                fn_call(WaitSetAttachmentId::tick(self, idx))
            };

            if let CallbackProgression::Stop = progression {
                result = WaitSetRunResult::StopRequest;
            }

            progression
        };

        fail!(from self,
                  when self.deadline_queue.missed_deadlines(call),
                  with WaitSetRunError::InternalError,
                  "{error_msg} since the missed deadlines could not be acquired.");

        Ok(result)
    }

    fn handle_all_attachments<F: FnMut(WaitSetAttachmentId<Service>) -> CallbackProgression>(
        &self,
        triggered_file_descriptors: &Vec<i32>,
        fn_call: &mut F,
        error_msg: &str,
    ) -> Result<WaitSetRunResult, WaitSetRunError> {
        // we need to reset the deadlines first, otherwise a long fn_call may extend the
        // deadline unintentionally
        let mut fd_and_deadline_queue_idx = Vec::with_capacity(triggered_file_descriptors.len());

        for fd in triggered_file_descriptors {
            fd_and_deadline_queue_idx.push((fd, self.reset_deadline(*fd)?));
        }

        // must be called after the deadlines have been reset, in the case that the
        // event has been received shortly before the deadline ended.

        match self.handle_deadlines(fn_call, error_msg)? {
            WaitSetRunResult::AllEventsHandled => (),
            v => return Ok(v),
        };

        for fd in triggered_file_descriptors {
            if let CallbackProgression::Stop = fn_call(WaitSetAttachmentId::notification(self, *fd))
            {
                return Ok(WaitSetRunResult::StopRequest);
            }
        }

        Ok(WaitSetRunResult::AllEventsHandled)
    }

    /// Attaches an object as notification to the [`WaitSet`]. Whenever an event is received on the
    /// object the [`WaitSet`] informs the user in [`WaitSet::wait_and_process()`] to handle the event.
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    pub fn attach_notification<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
    ) -> Result<WaitSetGuard<'waitset, 'attachment, Service>, WaitSetAttachmentError> {
        let reactor_guard = self.attach_to_reactor(attachment)?;
        self.attach()?;

        Ok(WaitSetGuard {
            waitset: self,
            guard_type: GuardType::Notification(reactor_guard),
        })
    }

    /// Attaches an object as deadline to the [`WaitSet`]. Whenever the event is received or the
    /// deadline is hit, the user is informed in [`WaitSet::wait_and_process()`].
    /// The object cannot be attached twice and the
    /// [`WaitSet::capacity()`] is limited by the underlying implementation.
    /// Whenever the object emits an event the deadline is reset by the [`WaitSet`].
    pub fn attach_deadline<'waitset, 'attachment, T: SynchronousMultiplexing + Debug>(
        &'waitset self,
        attachment: &'attachment T,
        deadline: Duration,
    ) -> Result<WaitSetGuard<'waitset, 'attachment, Service>, WaitSetAttachmentError> {
        let reactor_guard = self.attach_to_reactor(attachment)?;
        let deadline_queue_guard = self.attach_to_deadline_queue(deadline)?;

        let reactor_idx = unsafe { reactor_guard.file_descriptor().native_handle() };
        let deadline_idx = deadline_queue_guard.index();

        self.attachment_to_deadline
            .borrow_mut()
            .insert(reactor_idx, deadline_idx);
        self.deadline_to_attachment
            .borrow_mut()
            .insert(deadline_idx, reactor_idx);
        self.attach()?;

        Ok(WaitSetGuard {
            waitset: self,
            guard_type: GuardType::Deadline(reactor_guard, deadline_queue_guard),
        })
    }

    /// Attaches a tick event to the [`WaitSet`]. Whenever the timeout is reached the [`WaitSet`]
    /// informs the user in [`WaitSet::wait_and_process()`].
    pub fn attach_interval(
        &self,
        interval: Duration,
    ) -> Result<WaitSetGuard<Service>, WaitSetAttachmentError> {
        let deadline_queue_guard = self.attach_to_deadline_queue(interval)?;
        self.attach()?;

        Ok(WaitSetGuard {
            waitset: self,
            guard_type: GuardType::Tick(deadline_queue_guard),
        })
    }

    /// Waits until an event arrives on the [`WaitSet`], then collects all events by calling the
    /// provided `fn_call` callback with the corresponding [`WaitSetAttachmentId`]. In contrast
    /// to [`WaitSet::wait_and_process_once()`] it will never return until the user explicitly
    /// requests it by returning [`CallbackProgression::Stop`] or by receiving a signal.
    ///
    /// The provided callback must return [`CallbackProgression::Continue`] to continue the event
    /// processing and handle the next event or [`CallbackProgression::Stop`] to return from this
    /// call immediately. All unhandled events will be lost forever and the call will return
    /// [`WaitSetRunResult::StopRequest`].
    ///
    /// If an interrupt- (`SIGINT`) or a termination-signal (`SIGTERM`) was received, it will exit
    /// the loop and inform the user with [`WaitSetRunResult::Interrupt`] or
    /// [`WaitSetRunResult::TerminationRequest`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use iceoryx2::prelude::*;
    /// # use core::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let event = node.service_builder(&"MyEventName_1".try_into()?)
    /// #     .event()
    /// #     .open_or_create()?;
    ///
    /// # let mut listener = event.listener_builder().create()?;
    ///
    /// let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    /// # let guard = waitset.attach_notification(&listener)?;
    ///
    /// let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
    ///     if attachment_id.has_event_from(&guard) {
    ///         // when a certain event arrives we stop the event processing
    ///         // to terminate the process
    ///         CallbackProgression::Stop
    ///     } else {
    ///         CallbackProgression::Continue
    ///     }
    /// };
    ///
    /// waitset.wait_and_process(on_event)?;
    /// println!("goodbye");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn wait_and_process<F: FnMut(WaitSetAttachmentId<Service>) -> CallbackProgression>(
        &self,
        mut fn_call: F,
    ) -> Result<WaitSetRunResult, WaitSetRunError> {
        loop {
            match self.wait_and_process_once(&mut fn_call) {
                Ok(WaitSetRunResult::AllEventsHandled) => (),
                Ok(v) => return Ok(v),
                Err(e) => {
                    fail!(from self, with e,
                            "Unable to run in WaitSet::wait_and_process() loop since ({:?}) has occurred.", e);
                }
            }
        }
    }

    /// Waits until an event arrives on the [`WaitSet`], then collects all events by calling the
    /// provided `fn_call` callback with the corresponding [`WaitSetAttachmentId`] and then
    /// returns. This makes it ideal to be called in some kind of event-loop.
    ///
    /// The provided callback must return [`CallbackProgression::Continue`] to continue the event
    /// processing and handle the next event or [`CallbackProgression::Stop`] to return from this
    /// call immediately. All unhandled events will be lost forever and the call will return
    /// [`WaitSetRunResult::StopRequest`].
    ///
    /// If an interrupt- (`SIGINT`) or a termination-signal (`SIGTERM`) was received, it will exit
    /// the loop and inform the user with [`WaitSetRunResult::Interrupt`] or
    /// [`WaitSetRunResult::TerminationRequest`].
    ///
    /// When no signal was received and all events were handled, it will return
    /// [`WaitSetRunResult::AllEventsHandled`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use iceoryx2::prelude::*;
    /// # use core::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let event = node.service_builder(&"MyEventName_1".try_into()?)
    /// #     .event()
    /// #     .open_or_create()?;
    ///
    /// let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    ///
    /// let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
    ///     // do some event processing
    ///     CallbackProgression::Continue
    /// };
    ///
    /// // main event loop
    /// loop {
    ///     // blocks until an event arrives, handles all arrived events and then
    ///     // returns.
    ///     waitset.wait_and_process_once(on_event)?;
    ///     // do some event post processing
    ///     println!("handled events");
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn wait_and_process_once<F: FnMut(WaitSetAttachmentId<Service>) -> CallbackProgression>(
        &self,
        mut fn_call: F,
    ) -> Result<WaitSetRunResult, WaitSetRunError> {
        let msg = "Unable to call WaitSet::try_wait_and_process()";

        if SignalHandler::termination_requested() {
            return Ok(WaitSetRunResult::TerminationRequest);
        }

        if self.is_empty() {
            fail!(from self, with WaitSetRunError::NoAttachments,
                "{msg} since the WaitSet has no attachments, therefore the call would end up in a deadlock.");
        }

        let next_timeout = fail!(from self,
                                 when self.deadline_queue.duration_until_next_deadline(),
                                 with WaitSetRunError::InternalError,
                                 "{msg} since the next timeout could not be acquired.");

        let mut triggered_file_descriptors = vec![];
        let collect_triggered_fds = |fd: &FileDescriptor| {
            let fd = unsafe { fd.native_handle() };
            triggered_file_descriptors.push(fd);
        };

        // Collect all triggered file descriptors. We need to collect them first, then reset
        // the deadline and then call the callback, otherwise a long callback may destroy the
        // deadline contract.
        let reactor_wait_result = if self.deadline_queue.is_empty() {
            self.reactor.blocking_wait(collect_triggered_fds)
        } else {
            self.reactor.timed_wait(collect_triggered_fds, next_timeout)
        };

        match reactor_wait_result {
            Ok(0) => self.handle_deadlines(&mut fn_call, msg),
            Ok(_) => self.handle_all_attachments(&triggered_file_descriptors, &mut fn_call, msg),
            Err(ReactorWaitError::Interrupt) => Ok(WaitSetRunResult::Interrupt),
            Err(ReactorWaitError::InsufficientPermissions) => {
                fail!(from self, with WaitSetRunError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(ReactorWaitError::UnknownError) => {
                fail!(from self, with WaitSetRunError::InternalError,
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
        self.attachment_counter.load(Ordering::Relaxed)
    }

    /// Returns true if the [`WaitSet`] has no attachments, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    fn attach_to_deadline_queue(
        &self,
        timeout: Duration,
    ) -> Result<DeadlineQueueGuard, WaitSetAttachmentError> {
        let msg = "Unable to attach timeout to underlying Timer";

        match self.deadline_queue.add_deadline_interval(timeout) {
            Ok(guard) => Ok(guard),
            Err(e) => {
                fail!(from self, with WaitSetAttachmentError::InternalError,
                    "{msg} since the timeout could not be attached to the underlying deadline_queue due to ({:?}).", e);
            }
        }
    }
}
