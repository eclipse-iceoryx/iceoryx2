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
    error::{WaitSetAttachmentError, WaitSetRunError},
    file_descriptor::FileDescriptor,
    listener::{Listener, ListenerType},
    parc::Parc,
    signal_handling_mode::SignalHandlingMode,
    waitset_attachment_id::{WaitSetAttachmentId, WaitSetAttachmentIdType},
    waitset_guard::{StorageType, WaitSetGuard, WaitSetGuardType},
    waitset_run_result::WaitSetRunResult,
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

#[pymethods]
impl WaitSet {
    /// Attaches a `Listener` as notification to the `WaitSet`. Whenever an event is received on the
    /// object the `WaitSet` informs the user in `WaitSet::wait_and_process()` to handle the event.
    /// The object cannot be attached twice and the
    /// `WaitSet::capacity()` is limited by the underlying implementation.
    pub fn attach_notification(&self, attachment: &Listener) -> PyResult<WaitSetGuard> {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => {
                if let ListenerType::Ipc(Some(attachment)) = &attachment.0 {
                    let guard = v
                        .attach_notification(attachment.deref())
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Ipc(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe {
                            core::mem::transmute::<
                                iceoryx2::waitset::WaitSetGuard<'_, '_, crate::IpcService>,
                                iceoryx2::waitset::WaitSetGuard<
                                    'static,
                                    'static,
                                    crate::IpcService,
                                >,
                            >(guard)
                        }),
                        waitset: self.0.clone(),
                        _attachment: Some(attachment.clone()),
                    })))
                } else {
                    Err(WaitSetAttachmentError::new_err(
                        "The attachment has the wrong service type.",
                    ))
                }
            }
            WaitSetType::Local(v) => {
                if let ListenerType::Local(Some(attachment)) = &attachment.0 {
                    let guard = v
                        .attach_notification(attachment.deref())
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Local(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe {
                            core::mem::transmute::<
                                iceoryx2::waitset::WaitSetGuard<'_, '_, crate::LocalService>,
                                iceoryx2::waitset::WaitSetGuard<
                                    'static,
                                    'static,
                                    crate::LocalService,
                                >,
                            >(guard)
                        }),
                        waitset: self.0.clone(),
                        _attachment: Some(attachment.clone()),
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
                    guard: Some(unsafe {
                        core::mem::transmute::<
                            iceoryx2::waitset::WaitSetGuard<'_, '_, crate::IpcService>,
                            iceoryx2::waitset::WaitSetGuard<'static, 'static, crate::IpcService>,
                        >(guard)
                    }),
                    waitset: self.0.clone(),
                    _attachment: Some(attachment.0.clone()),
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
                    guard: Some(unsafe {
                        core::mem::transmute::<
                            iceoryx2::waitset::WaitSetGuard<'_, '_, crate::LocalService>,
                            iceoryx2::waitset::WaitSetGuard<'static, 'static, crate::LocalService>,
                        >(guard)
                    }),
                    waitset: self.0.clone(),
                    _attachment: Some(attachment.0.clone()),
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
                if let ListenerType::Ipc(Some(attachment)) = &attachment.0 {
                    let guard = v
                        .attach_deadline(attachment.deref(), deadline.0)
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Ipc(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe {
                            core::mem::transmute::<
                                iceoryx2::waitset::WaitSetGuard<'_, '_, crate::IpcService>,
                                iceoryx2::waitset::WaitSetGuard<
                                    'static,
                                    'static,
                                    crate::IpcService,
                                >,
                            >(guard)
                        }),
                        waitset: self.0.clone(),
                        _attachment: Some(attachment.clone()),
                    })))
                } else {
                    Err(WaitSetAttachmentError::new_err(
                        "The attachment has the wrong service type.",
                    ))
                }
            }
            WaitSetType::Local(v) => {
                if let ListenerType::Local(Some(attachment)) = &attachment.0 {
                    let guard = v
                        .attach_deadline(attachment.deref(), deadline.0)
                        .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                    Ok(WaitSetGuard(WaitSetGuardType::Local(StorageType {
                        // safe since the waitset arc and the attachment arc become a member of the
                        // guard and therefore the waitset and the attachment always lives at least
                        // as long as the guard
                        guard: Some(unsafe {
                            core::mem::transmute::<
                                iceoryx2::waitset::WaitSetGuard<'_, '_, crate::LocalService>,
                                iceoryx2::waitset::WaitSetGuard<
                                    'static,
                                    'static,
                                    crate::LocalService,
                                >,
                            >(guard)
                        }),
                        waitset: self.0.clone(),
                        _attachment: Some(attachment.clone()),
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
                    guard: Some(unsafe {
                        core::mem::transmute::<
                            iceoryx2::waitset::WaitSetGuard<'_, '_, crate::IpcService>,
                            iceoryx2::waitset::WaitSetGuard<'static, 'static, crate::IpcService>,
                        >(guard)
                    }),
                    waitset: self.0.clone(),
                    _attachment: Some(attachment.0.clone()),
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
                    guard: Some(unsafe {
                        core::mem::transmute::<
                            iceoryx2::waitset::WaitSetGuard<'_, '_, crate::LocalService>,
                            iceoryx2::waitset::WaitSetGuard<'static, 'static, crate::LocalService>,
                        >(guard)
                    }),
                    waitset: self.0.clone(),
                    _attachment: Some(attachment.0.clone()),
                })))
            }
        }
    }

    /// Attaches a tick event to the `WaitSet`. Whenever the timeout is reached the `WaitSet`
    /// informs the user in `WaitSet::wait_and_process()`.
    pub fn attach_interval(&self, interval: &Duration) -> PyResult<WaitSetGuard> {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => {
                let guard = v
                    .attach_interval(interval.0)
                    .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                Ok(WaitSetGuard(WaitSetGuardType::Ipc(StorageType {
                    // safe since the waitset arc becomes a member of the guard and therefore the
                    // waitset lives at least as long as the guard
                    guard: Some(unsafe {
                        core::mem::transmute::<
                            iceoryx2::waitset::WaitSetGuard<'_, '_, crate::IpcService>,
                            iceoryx2::waitset::WaitSetGuard<'static, 'static, crate::IpcService>,
                        >(guard)
                    }),
                    waitset: self.0.clone(),
                    _attachment: None,
                })))
            }
            WaitSetType::Local(v) => {
                let guard = v
                    .attach_interval(interval.0)
                    .map_err(|e| WaitSetAttachmentError::new_err(format!("{e:?}")))?;
                Ok(WaitSetGuard(WaitSetGuardType::Local(StorageType {
                    // safe since the waitset arc becomes a member of the guard and therefore the
                    // waitset lives at least as long as the guard
                    guard: Some(unsafe {
                        core::mem::transmute::<
                            iceoryx2::waitset::WaitSetGuard<'_, '_, crate::LocalService>,
                            iceoryx2::waitset::WaitSetGuard<'static, 'static, crate::LocalService>,
                        >(guard)
                    }),
                    waitset: self.0.clone(),
                    _attachment: None,
                })))
            }
        }
    }

    /// Waits until an event arrives on the `WaitSet`, then collects the events corresponding
    /// `WaitSetAttachmentId` in a vector and returns it.
    ///
    /// If an interrupt- (`SIGINT`) or a termination-signal (`SIGTERM`) was received, it will exit
    /// the loop and inform the user with [`WaitSetRunResult::Interrupt`] or
    /// [`WaitSetRunResult::TerminationRequest`].
    pub fn wait_and_process(&self) -> PyResult<(Vec<WaitSetAttachmentId>, WaitSetRunResult)> {
        let mut ret_val = vec![];
        let result = match &*self.0.lock() {
            WaitSetType::Ipc(v) => v
                .wait_and_process_once(|v| {
                    ret_val.push(WaitSetAttachmentId(WaitSetAttachmentIdType::Ipc(v)));
                    iceoryx2::prelude::CallbackProgression::Continue
                })
                .map_err(|e| WaitSetRunError::new_err(format!("{e:?}")))?,
            WaitSetType::Local(v) => v
                .wait_and_process_once(|v| {
                    ret_val.push(WaitSetAttachmentId(WaitSetAttachmentIdType::Local(v)));
                    iceoryx2::prelude::CallbackProgression::Continue
                })
                .map_err(|e| WaitSetRunError::new_err(format!("{e:?}")))?,
        };

        Ok((ret_val, result.into()))
    }

    /// Waits until an event arrives on the `WaitSet` or the provided timeout has passed, then
    /// collects the events corresponding `WaitSetAttachmentId` in a vector and returns it.
    ///
    /// If an interrupt- (`SIGINT`) or a termination-signal (`SIGTERM`) was received, it will exit
    /// the loop and inform the user with [`WaitSetRunResult::Interrupt`] or
    /// [`WaitSetRunResult::TerminationRequest`].
    pub fn wait_and_process_with_timeout(
        &self,
        timeout: &Duration,
    ) -> PyResult<(Vec<WaitSetAttachmentId>, WaitSetRunResult)> {
        let mut ret_val = vec![];
        let result = match &*self.0.lock() {
            WaitSetType::Ipc(v) => v
                .wait_and_process_once_with_timeout(
                    |v| {
                        ret_val.push(WaitSetAttachmentId(WaitSetAttachmentIdType::Ipc(v)));
                        iceoryx2::prelude::CallbackProgression::Continue
                    },
                    timeout.0,
                )
                .map_err(|e| WaitSetRunError::new_err(format!("{e:?}")))?,
            WaitSetType::Local(v) => v
                .wait_and_process_once_with_timeout(
                    |v| {
                        ret_val.push(WaitSetAttachmentId(WaitSetAttachmentIdType::Local(v)));
                        iceoryx2::prelude::CallbackProgression::Continue
                    },
                    timeout.0,
                )
                .map_err(|e| WaitSetRunError::new_err(format!("{e:?}")))?,
        };

        Ok((ret_val, result.into()))
    }

    #[getter]
    /// Returns the capacity of the `WaitSet`
    pub fn capacity(&self) -> usize {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => v.capacity(),
            WaitSetType::Local(v) => v.capacity(),
        }
    }

    #[getter]
    /// Returns the number of attachments.
    pub fn len(&self) -> usize {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => v.len(),
            WaitSetType::Local(v) => v.len(),
        }
    }

    #[getter]
    /// Returns true if the `WaitSet` has no attachments, otherwise false.
    pub fn is_empty(&self) -> bool {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => v.is_empty(),
            WaitSetType::Local(v) => v.is_empty(),
        }
    }

    #[getter]
    /// Returns the `SignalHandlingMode` with which the `WaitSet` was created.
    pub fn signal_handling_mode(&self) -> SignalHandlingMode {
        match &*self.0.lock() {
            WaitSetType::Ipc(v) => v.signal_handling_mode().into(),
            WaitSetType::Local(v) => v.signal_handling_mode().into(),
        }
    }
}
