// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

//! A [`UserTrigger`](crate::user_trigger::UserTrigger) wakes up a
//! [`WaitSet`](crate::waitset::WaitSet) from any thread of the process. It is created by a
//! [`Node`](crate::node::Node) with
//! [`Node::create_user_trigger()`](crate::node::Node::create_user_trigger) and attached to a
//! [`WaitSet`](crate::waitset::WaitSet) like a [`Listener`](crate::port::listener::Listener).
//! Every [`UserTrigger::notify()`](crate::user_trigger::UserTrigger::notify) wakes up the
//! [`WaitSet`](crate::waitset::WaitSet) until
//! [`UserTrigger::reset()`](crate::user_trigger::UserTrigger::reset) consumes the
//! notifications. Notifications are not counted, any number of them before a reset is
//! reported as one.
//!
//! # Example
//!
//! ```no_run
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let waitset = node.waitset_builder().create()?;
//!
//! let user_trigger = node.create_user_trigger()?;
//! let guard = waitset.attach_notification(&user_trigger)?;
//!
//! std::thread::scope(|s| {
//!     s.spawn(|| user_trigger.notify());
//!
//!     let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
//!         if attachment_id.has_event_from(&guard) && user_trigger.reset().unwrap_or(false) {
//!             println!("the user trigger was notified");
//!         }
//!         CallbackProgression::Continue
//!     };
//!     waitset.wait_and_process_once(on_event)
//! })?;
//! # Ok(())
//! # }
//! ```

use iceoryx2_bb_posix::file_descriptor::{FileDescriptor, FileDescriptorBased};
use iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing;
use iceoryx2_cal::trigger::{Activator, Observer, Trigger, TriggerBuilder, TriggerCreateError};
use iceoryx2_log::fail;

use crate::service;

/// Defines the failures that can occur when a [`UserTrigger`] is created with
/// [`Node::create_user_trigger()`](crate::node::Node::create_user_trigger).
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UserTriggerCreateError {
    /// The process or the system ran out of the resources a [`UserTrigger`] requires.
    InsufficientResources,
    /// An unexpected failure occurred.
    InternalError,
}

impl core::fmt::Display for UserTriggerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UserTriggerCreateError::{self:?}")
    }
}

impl core::error::Error for UserTriggerCreateError {}

/// Defines the failures that can occur in [`UserTrigger::notify()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UserTriggerNotifyError {
    /// An unexpected failure occurred.
    InternalError,
}

impl core::fmt::Display for UserTriggerNotifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UserTriggerNotifyError::{self:?}")
    }
}

impl core::error::Error for UserTriggerNotifyError {}

/// Defines the failures that can occur in [`UserTrigger::reset()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UserTriggerResetError {
    /// An unexpected failure occurred.
    InternalError,
}

impl core::fmt::Display for UserTriggerResetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UserTriggerResetError::{self:?}")
    }
}

impl core::error::Error for UserTriggerResetError {}

/// Wakes up a [`WaitSet`](crate::waitset::WaitSet) from any thread of the process. See the
/// [module documentation](crate::user_trigger) for an example.
#[derive(Debug)]
pub struct UserTrigger<Service: service::Service> {
    activator: <Service::LocalTrigger as Trigger>::Activator,
    observer: <Service::LocalTrigger as Trigger>::Observer,
}

impl<Service: service::Service> FileDescriptorBased for UserTrigger<Service> {
    fn file_descriptor(&self) -> &FileDescriptor {
        self.observer.file_descriptor()
    }
}

impl<Service: service::Service> SynchronousMultiplexing for UserTrigger<Service> {}

impl<Service: service::Service> UserTrigger<Service> {
    pub(crate) fn new() -> Result<Self, UserTriggerCreateError> {
        let origin = "UserTrigger::new()";
        let msg = "Unable to create UserTrigger";
        match <Service::LocalTrigger as Trigger>::Builder::new().create() {
            Ok((activator, observer)) => Ok(Self {
                activator,
                observer,
            }),
            Err(TriggerCreateError::InsufficientResources) => {
                fail!(from origin, with UserTriggerCreateError::InsufficientResources,
                    "{msg} due to insufficient resources.");
            }
            Err(TriggerCreateError::InternalError) => {
                fail!(from origin, with UserTriggerCreateError::InternalError,
                    "{msg} due to an internal failure.");
            }
        }
    }

    /// Wakes up every [`WaitSet`](crate::waitset::WaitSet) the [`UserTrigger`] is attached
    /// to, until it is reset with [`UserTrigger::reset()`]. Notifying an already notified
    /// [`UserTrigger`] succeeds.
    pub fn notify(&self) -> Result<(), UserTriggerNotifyError> {
        fail!(from self, when self.activator.activate(),
            with UserTriggerNotifyError::InternalError,
            "Unable to notify the UserTrigger due to an internal failure.");
        Ok(())
    }

    /// Consumes all notifications. Returns `true` when the [`UserTrigger`] was notified since
    /// the last reset, otherwise `false`.
    pub fn reset(&self) -> Result<bool, UserTriggerResetError> {
        Ok(fail!(from self, when self.observer.reset(),
            with UserTriggerResetError::InternalError,
            "Unable to reset the UserTrigger due to an internal failure."))
    }
}
