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

use core::ops::Deref;

use pyo3::prelude::*;

use crate::{
    duration::Duration,
    error::WaitSetAttachmentError,
    file_descriptor::FileDescriptor,
    listener::{Listener, ListenerType},
    parc::Parc,
    waitset_guard::{StorageType, WaitSetGuard, WaitSetGuardType},
};

pub(crate) enum WaitSetType {
    Ipc(iceoryx2::waitset::WaitSet<crate::IpcService>),
    Local(iceoryx2::waitset::WaitSet<crate::LocalService>),
}

#[pyclass]
/// The `WaitSet` implements a reactor pattern and allows to wait on multiple events in one
/// single call `WaitSet::wait_and_process()` until a interrupt or termination signal was received.
///
/// The `Listener` can be attached as well as sockets or anything else that is a `FileDescriptor`.
///
/// Can be created via the `WaitSetBuilder`.
pub struct WaitSet(pub(crate) Parc<WaitSetType>);

impl WaitSet {
    /// Attaches a `Listener` as notification to the `WaitSet`. Whenever an event is received on the
    /// object the `WaitSet` informs the user in `WaitSet::wait_and_process()` to handle the event.
    /// The object cannot be attached twice and the
    /// `WaitSet::capacity()` is limited by the underlying implementation.
    pub fn attach_notification(&self, attachment: &Listener) -> PyResult<WaitSetGuard> {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => {
                if let ListenerType::Ipc(attachment) = &attachment.0 {
                    let guard = v
                        .attach_notification(attachment.deref())
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Ipc(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe { core::mem::transmute(guard) }),
                        waitset: self.0.clone(),
                        _attachment: attachment.clone(),
                    })))
                } else {
                    Err(WaitSetAttachmentError::new_err(
                        "The attachment has the wrong service type.",
                    ))
                }
            }
            WaitSetType::Local(v) => {
                if let ListenerType::Local(attachment) = &attachment.0 {
                    let guard = v
                        .attach_notification(attachment.deref())
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Local(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe { core::mem::transmute(guard) }),
                        waitset: self.0.clone(),
                        _attachment: attachment.clone(),
                    })))
                } else {
                    Err(WaitSetAttachmentError::new_err(
                        "The attachment has the wrong service type.",
                    ))
                }
            }
        }
    }

    /// Attaches a `FileDescriptor` as notification to the `WaitSet`. Whenever an event is received on the
    /// object the `WaitSet` informs the user in `WaitSet::wait_and_process()` to handle the event.
    /// The object cannot be attached twice and the
    /// `WaitSet::capacity()` is limited by the underlying implementation.
    pub fn attach_notification_fd(&self, attachment: &FileDescriptor) -> PyResult<WaitSetGuard> {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => {
                let guard = v
                    .attach_notification(attachment)
                    .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                Ok(WaitSetGuard(WaitSetGuardType::Ipc(StorageType {
                    // safe since the waitset arc and the attachment arc become a member of the
                    // guard and therefore the waitset and the attachment always lives at least
                    // as long as the guard
                    guard: Some(unsafe { core::mem::transmute(guard) }),
                    waitset: self.0.clone(),
                    _attachment: attachment.0.clone(),
                })))
            }
            WaitSetType::Local(v) => {
                let guard = v
                    .attach_notification(attachment)
                    .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                Ok(WaitSetGuard(WaitSetGuardType::Local(StorageType {
                    // safe since the waitset arc and the attachment arc become a member of the
                    // guard and therefore the waitset and the attachment always lives at least
                    // as long as the guard
                    guard: Some(unsafe { core::mem::transmute(guard) }),
                    waitset: self.0.clone(),
                    _attachment: attachment.0.clone(),
                })))
            }
        }
    }

    /// Attaches a `Listener` as deadline to the `WaitSet`. Whenever the event is received or the
    /// deadline is hit, the user is informed in `WaitSet::wait_and_process()`.
    /// The object cannot be attached twice and the
    /// `WaitSet::capacity()` is limited by the underlying implementation.
    /// Whenever the object emits an event the deadline is reset by the `WaitSet`.
    pub fn attach_deadline(
        &self,
        attachment: &Listener,
        deadline: &Duration,
    ) -> PyResult<WaitSetGuard> {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => {
                if let ListenerType::Ipc(attachment) = &attachment.0 {
                    let guard = v
                        .attach_deadline(attachment.deref(), deadline.0)
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Ipc(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe { core::mem::transmute(guard) }),
                        waitset: self.0.clone(),
                        _attachment: attachment.clone(),
                    })))
                } else {
                    Err(WaitSetAttachmentError::new_err(
                        "The attachment has the wrong service type.",
                    ))
                }
            }
            WaitSetType::Local(v) => {
                if let ListenerType::Local(attachment) = &attachment.0 {
                    let guard = v
                        .attach_deadline(attachment.deref(), deadline.0)
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Local(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe { core::mem::transmute(guard) }),
                        waitset: self.0.clone(),
                        _attachment: attachment.clone(),
                    })))
                } else {
                    Err(WaitSetAttachmentError::new_err(
                        "The attachment has the wrong service type.",
                    ))
                }
            }
        }
    }

    /// Attaches a `FileDescriptor` as deadline to the `WaitSet`. Whenever the event is received or
    /// the deadline is hit, the user is informed in `WaitSet::wait_and_process()`.
    /// The object cannot be attached twice and the
    /// `WaitSet::capacity()` is limited by the underlying implementation.
    /// Whenever the object emits an event the deadline is reset by the `WaitSet`.
    pub fn attach_deadline_fd(
        &self,
        attachment: &FileDescriptor,
        deadline: &Duration,
    ) -> PyResult<WaitSetGuard> {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => {
                let guard = v
                    .attach_deadline(attachment, deadline.0)
                    .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                Ok(WaitSetGuard(WaitSetGuardType::Ipc(StorageType {
                    // safe since the waitset arc and the attachment arc become a member of the
                    // guard and therefore the waitset and the attachment always lives at least
                    // as long as the guard
                    guard: Some(unsafe { core::mem::transmute(guard) }),
                    waitset: self.0.clone(),
                    _attachment: attachment.0.clone(),
                })))
            }
            WaitSetType::Local(v) => {
                let guard = v
                    .attach_deadline(attachment, deadline.0)
                    .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                Ok(WaitSetGuard(WaitSetGuardType::Local(StorageType {
                    // safe since the waitset arc and the attachment arc become a member of the
                    // guard and therefore the waitset and the attachment always lives at least
                    // as long as the guard
                    guard: Some(unsafe { core::mem::transmute(guard) }),
                    waitset: self.0.clone(),
                    _attachment: attachment.0.clone(),
                })))
            }
        }
    }
}
