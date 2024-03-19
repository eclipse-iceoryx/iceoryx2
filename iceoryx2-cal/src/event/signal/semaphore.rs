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

use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::{
    semaphore::{
        SemaphoreInterface, UnnamedSemaphore, UnnamedSemaphoreBuilder, UnnamedSemaphoreHandle,
    },
    unmovable_ipc_handle::IpcCapable,
};

use crate::event::{ListenerCreateError, ListenerWaitError, NotifierNotifyError};

use super::SignalMechanism;

#[derive(Debug)]
pub struct Semaphore {
    handle: UnnamedSemaphoreHandle,
}

impl Semaphore {
    fn semaphore(&self) -> UnnamedSemaphore<'_> {
        fatal_panic!(from self,
            when UnnamedSemaphore::from_ipc_handle(&self.handle),
            "This should never happen! Failed to acquire semaphore handle.")
    }
}

impl SignalMechanism for Semaphore {
    fn new() -> Self {
        Self {
            handle: UnnamedSemaphoreHandle::new(),
        }
    }

    fn init(&mut self) -> Result<(), ListenerCreateError> {
        fail!(from self, when UnnamedSemaphoreBuilder::new()
            .is_interprocess_capable(true)
            .create(&self.handle),
            with ListenerCreateError::InternalFailure,
            "Unable to initialize underlying semaphore due to an internal failure.");

        Ok(())
    }

    fn notify(&self) -> Result<(), NotifierNotifyError> {
        fail!(from self, when self.semaphore().post(),
            with NotifierNotifyError::InternalFailure,
            "Failed to increment underlying semaphore.");
        Ok(())
    }

    fn try_wait(&self) -> Result<(), ListenerWaitError> {
        fail!(from self, when self.semaphore().try_wait(),
            with ListenerWaitError::InternalFailure,
            "Failed to dedcrement underlying semaphore.");
        Ok(())
    }

    fn timed_wait(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), crate::event::ListenerWaitError> {
        fail!(from self, when self.semaphore().timed_wait(timeout),
            with ListenerWaitError::InternalFailure,
            "Failed to dedcrement underlying semaphore.");
        Ok(())
    }

    fn blocking_wait(&self) -> Result<(), crate::event::ListenerWaitError> {
        fail!(from self, when self.semaphore().wait(),
            with ListenerWaitError::InternalFailure,
            "Failed to dedcrement underlying semaphore.");
        Ok(())
    }
}
