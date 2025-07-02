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

use pyo3::prelude::*;

use crate::waitset_guard::{WaitSetGuard, WaitSetGuardType};

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum WaitSetAttachmentIdType {
    Ipc(iceoryx2::prelude::WaitSetAttachmentId<crate::IpcService>),
    Local(iceoryx2::prelude::WaitSetAttachmentId<crate::LocalService>),
}

#[derive(PartialEq, Eq, Hash)]
#[pyclass(eq, hash, frozen)]
/// Represents an attachment to the `WaitSet`
pub struct WaitSetAttachmentId(pub(crate) WaitSetAttachmentIdType);

#[pymethods]
impl WaitSetAttachmentId {
    #[staticmethod]
    /// Creates an `WaitSetAttachmentId` from a `WaitSetGuard` that was returned via
    /// `WaitSet::attach_interval()`, `WaitSet::attach_notification()` or
    /// `WaitSet::attach_deadline()`.
    pub fn from_guard(guard: &WaitSetGuard) -> Self {
        match &guard.0 {
            WaitSetGuardType::Ipc(guard) => WaitSetAttachmentId(WaitSetAttachmentIdType::Ipc(
                iceoryx2::prelude::WaitSetAttachmentId::<crate::IpcService>::from_guard(
                    guard.guard.as_ref().unwrap(),
                ),
            )),
            WaitSetGuardType::Local(guard) => WaitSetAttachmentId(WaitSetAttachmentIdType::Local(
                iceoryx2::prelude::WaitSetAttachmentId::<crate::LocalService>::from_guard(
                    guard.guard.as_ref().unwrap(),
                ),
            )),
        }
    }

    /// Returns true if an event was emitted from a notification or deadline attachment
    /// corresponding to `WaitSetGuard`.
    pub fn has_event_from(&self, other: &WaitSetGuard) -> bool {
        match &self.0 {
            WaitSetAttachmentIdType::Ipc(v) => {
                if let WaitSetGuardType::Ipc(guard) = &other.0 {
                    if let Some(guard) = &guard.guard {
                        return v.has_event_from(guard);
                    }
                }
                false
            }
            WaitSetAttachmentIdType::Local(v) => {
                if let WaitSetGuardType::Local(guard) = &other.0 {
                    if let Some(guard) = &guard.guard {
                        return v.has_event_from(guard);
                    }
                }
                false
            }
        }
    }

    /// Returns true if the deadline for the attachment corresponding to `WaitSetGuard` was missed.
    pub fn has_missed_deadline(&self, other: &WaitSetGuard) -> bool {
        match &self.0 {
            WaitSetAttachmentIdType::Ipc(v) => {
                if let WaitSetGuardType::Ipc(guard) = &other.0 {
                    if let Some(guard) = &guard.guard {
                        return v.has_missed_deadline(guard);
                    }
                }
                false
            }
            WaitSetAttachmentIdType::Local(v) => {
                if let WaitSetGuardType::Local(guard) = &other.0 {
                    if let Some(guard) = &guard.guard {
                        return v.has_missed_deadline(guard);
                    }
                }
                false
            }
        }
    }
}
