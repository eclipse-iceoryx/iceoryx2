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

use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::{
    mutex::{Handle, IpcCapable},
    semaphore::{
        SemaphoreInterface, SemaphoreTimedWaitError, SemaphoreWaitError, UnnamedSemaphore,
        UnnamedSemaphoreBuilder, UnnamedSemaphoreHandle,
    },
};

use crate::event::{ListenerCreateError, ListenerWaitError, NotifierNotifyError};

use super::SignalMechanism;

#[derive(Debug)]
pub struct Semaphore {
    handle: UnnamedSemaphoreHandle,
}

impl Semaphore {
    fn semaphore(&self) -> UnnamedSemaphore<'_> {
        unsafe { UnnamedSemaphore::from_ipc_handle(&self.handle) }
    }
}

impl SignalMechanism for Semaphore {
    fn new() -> Self {
        Self {
            handle: UnnamedSemaphoreHandle::new(),
        }
    }

    unsafe fn init(&mut self) -> Result<(), ListenerCreateError> {
        fail!(from self, when UnnamedSemaphoreBuilder::new()
            .is_interprocess_capable(true)
            .create(&self.handle),
            with ListenerCreateError::InternalFailure,
            "Unable to initialize underlying semaphore due to an internal failure.");

        Ok(())
    }

    unsafe fn notify(&self) -> Result<(), NotifierNotifyError> {
        fail!(from self, when self.semaphore().post(),
            with NotifierNotifyError::InternalFailure,
            "Failed to increment underlying semaphore.");
        Ok(())
    }

    unsafe fn try_wait(&self) -> Result<bool, ListenerWaitError> {
        Ok(fail!(from self, when self.semaphore().try_wait(),
            with ListenerWaitError::InternalFailure,
            "Failed to dedcrement underlying semaphore."))
    }

    unsafe fn timed_wait(
        &self,
        timeout: core::time::Duration,
    ) -> Result<bool, crate::event::ListenerWaitError> {
        let msg = "Failed to decrement underlying semaphore with timeout";
        match self.semaphore().timed_wait(timeout) {
            Ok(state) => Ok(state),
            Err(SemaphoreTimedWaitError::SemaphoreWaitError(SemaphoreWaitError::Interrupt)) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{} since an interrupt signal was received.", msg );
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{} due to an internal failure ({:?}).", msg, e );
            }
        }
    }

    unsafe fn blocking_wait(&self) -> Result<(), crate::event::ListenerWaitError> {
        let msg = "Failed to decrement underlying semaphore in blocking mode";
        match self.semaphore().blocking_wait() {
            Ok(()) => Ok(()),
            Err(SemaphoreWaitError::Interrupt) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{} since an interrupt signal was received.", msg );
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{} due to an internal failure ({:?}).", msg, e );
            }
        }
    }
}
