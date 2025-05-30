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

use core::time::Duration;
pub use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::{debug, fail, fatal_panic};
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptorBased,
    file_descriptor_set::SynchronousMultiplexing,
    mutex::{Handle, Mutex, MutexBuilder, MutexHandle},
    socket_pair::{
        StreamingSocket, StreamingSocketDuplicateError, StreamingSocketPairCreationError,
        StreamingSocketPairReceiveError, StreamingSocketPairSendError,
    },
};
pub use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath, path::Path};
use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::named_concept::{
    NamedConceptConfiguration, NamedConceptDoesExistError, NamedConceptListError,
    NamedConceptRemoveError,
};

use super::{
    Event, ListenerCreateError, ListenerWaitError, NamedConcept, NamedConceptBuilder,
    NamedConceptMgmt, NotifierCreateError, NotifierNotifyError, TriggerId,
};

const MAX_BATCH_SIZE: usize = 512;

#[derive(Debug)]
struct StorageEntry {
    notifier: StreamingSocket,
}

static PROCESS_LOCAL_MTX_HANDLE: Lazy<MutexHandle<HashMap<FilePath, StorageEntry>>> =
    Lazy::new(MutexHandle::new);
static PROCESS_LOCAL_STORAGE: Lazy<Mutex<HashMap<FilePath, StorageEntry>>> = Lazy::new(|| {
    let result = MutexBuilder::new()
        .is_interprocess_capable(false)
        .create(HashMap::new(), &PROCESS_LOCAL_MTX_HANDLE);

    if result.is_err() {
        fatal_panic!(from "PROCESS_LOCAL_STORAGE", "Failed to create global dynamic storage");
    }

    result.unwrap()
});

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    prefix: FileName,
    path: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            path: EventImpl::default_path_hint(),
            suffix: EventImpl::default_suffix(),
            prefix: EventImpl::default_prefix(),
        }
    }
}

impl NamedConceptConfiguration for Configuration {
    fn prefix(mut self, value: &FileName) -> Self {
        self.prefix = value.clone();
        self
    }

    fn get_prefix(&self) -> &FileName {
        &self.prefix
    }

    fn suffix(mut self, value: &FileName) -> Self {
        self.suffix = value.clone();
        self
    }

    fn path_hint(mut self, value: &Path) -> Self {
        self.path = value.clone();
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug)]
pub struct EventImpl {}

impl NamedConceptMgmt for EventImpl {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError> {
        let msg = "Unable to check if event::process_local_socketpair exists";
        let origin = "event::process_local_socketpair::Event::does_exist_cfg()";

        let guard = fail!(from origin, when PROCESS_LOCAL_STORAGE.lock(),
                        with NamedConceptDoesExistError::InternalError,
                        "{} since the lock could not be acquired.", msg);

        match guard.get(&cfg.path_for(name)) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn list_cfg(cfg: &Self::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        let msg = "Unable to list all event::process_local_socketpairs";
        let origin = "event::process_local_socketpair::Event::list_cfg()";

        let guard = fail!(from origin, when PROCESS_LOCAL_STORAGE.lock(),
                                with NamedConceptListError::InternalError,
                                "{} since the lock could not be acquired.", msg);

        let mut result = vec![];
        for storage_name in guard.keys() {
            if let Some(v) = cfg.extract_name_from_path(storage_name) {
                result.push(v);
            }
        }

        Ok(result)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, NamedConceptRemoveError> {
        let storage_name = cfg.path_for(name);

        let msg = "Unable to remove dynamic_storage::process_local";
        let origin = "dynamic_storage::process_local::Storage::remove_cfg()";

        let mut guard = fail!(from origin, when PROCESS_LOCAL_STORAGE.lock(),
                                with NamedConceptRemoveError::InternalError,
                                "{} since the lock could not be acquired.", msg);

        let entry = guard.get_mut(&storage_name);
        if entry.is_none() {
            return Ok(false);
        }

        Ok(guard.remove(&storage_name).is_some())
    }

    fn remove_path_hint(
        _value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Ok(())
    }
}

impl crate::event::Event for EventImpl {
    type Notifier = Notifier;
    type Listener = Listener;
    type NotifierBuilder = NotifierBuilder;
    type ListenerBuilder = ListenerBuilder;
}

#[derive(Debug)]
pub struct Notifier {
    socket: StreamingSocket,
    name: FileName,
}

impl NamedConcept for Notifier {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl crate::event::Notifier for Notifier {
    fn notify(&self, id: TriggerId) -> Result<(), NotifierNotifyError> {
        let msg = "Unable to send notification";
        let buffer = unsafe {
            core::slice::from_raw_parts(
                (&id as *const TriggerId) as *const u8,
                core::mem::size_of::<TriggerId>(),
            )
        };
        match self.socket.try_send(buffer) {
            Ok(number_of_bytes) => {
                if number_of_bytes == 0 {
                    fail!(from self, with NotifierNotifyError::FailedToDeliverSignal,
                        "{msg} {id:?} since the listener buffer seems to be full.");
                } else if number_of_bytes == core::mem::size_of::<TriggerId>() {
                    Ok(())
                } else {
                    fatal_panic!(from self, "This should never happen! {msg} {id:?} could be sent only partially.");
                }
            }
            Err(StreamingSocketPairSendError::Interrupt) => {
                fail!(from self, with NotifierNotifyError::Interrupt,
                    "{msg} since an interrupt signal was received.");
            }
            Err(StreamingSocketPairSendError::ConnectionReset)
            | Err(StreamingSocketPairSendError::Disconnected) => {
                fail!(from self, with NotifierNotifyError::Disconnected,
                    "{msg} since the corresponding listener disconnected.");
            }
            Err(e) => {
                fail!(from self, with NotifierNotifyError::InternalFailure,
                    "{msg} due to an unknown failure ({:?}).", e);
            }
        }
    }
}

#[derive(Debug)]
pub struct NotifierBuilder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<EventImpl> for NotifierBuilder {
    fn new(name: &FileName) -> Self {
        Self {
            name: name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl crate::event::NotifierBuilder<EventImpl> for NotifierBuilder {
    fn timeout(self, _timeout: Duration) -> Self {
        self
    }

    fn open(self) -> Result<Notifier, NotifierCreateError> {
        let msg = "Failed to open Notifier";
        let full_path = self.config.path_for(&self.name);

        let guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with NotifierCreateError::InternalFailure,
            "{msg} due to a failure while acquiring the lock.");

        match guard.get(&full_path) {
            Some(entry) => match entry.notifier.duplicate() {
                Ok(socket) => Ok(Notifier {
                    name: self.name,
                    socket,
                }),
                Err(StreamingSocketDuplicateError::Interrupt) => {
                    fail!(from self, with NotifierCreateError::Interrupt,
                        "{msg} since an interrupt signal was received.");
                }
                Err(e) => {
                    fail!(from self, with NotifierCreateError::InternalFailure,
                        "{msg} due to an unknown failure ({:?}).", e);
                }
            },
            None => {
                fail!(from self, with NotifierCreateError::DoesNotExist,
                    "{msg} since the event does not exist.");
            }
        }
    }
}

#[derive(Debug)]
pub struct Listener {
    name: FileName,
    socket: StreamingSocket,
    config: Configuration,
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Err(e) = unsafe { EventImpl::remove_cfg(&self.name, &self.config) } {
            debug!(from self, "Unable to cleanup event after the Listener was dropped ({:?}).", e);
        }
    }
}

impl FileDescriptorBased for Listener {
    fn file_descriptor(&self) -> &iceoryx2_bb_posix::file_descriptor::FileDescriptor {
        self.socket.file_descriptor()
    }
}

impl SynchronousMultiplexing for Listener {}

impl NamedConcept for Listener {
    fn name(&self) -> &FileName {
        &self.name
    }
}

impl Listener {
    fn wait_one_impl<
        WaitCall: FnMut(&mut [u8]) -> Result<usize, StreamingSocketPairReceiveError>,
    >(
        &self,
        mut waitcall: WaitCall,
        msg: &str,
    ) -> Result<Option<TriggerId>, ListenerWaitError> {
        let trigger_id_size = core::mem::size_of::<TriggerId>();
        let mut trigger_id = TriggerId::new(0);
        let raw_trigger_id = unsafe {
            core::slice::from_raw_parts_mut(
                ((&mut trigger_id) as *mut TriggerId) as *mut u8,
                trigger_id_size,
            )
        };

        match waitcall(raw_trigger_id) {
            Ok(number_of_bytes) => {
                if number_of_bytes == 0 {
                    Ok(None)
                } else if number_of_bytes == trigger_id_size {
                    Ok(Some(trigger_id))
                } else {
                    fail!(from self, with ListenerWaitError::ContractViolation,
                    "{msg} due to a contract violation. Expected to receive {} bytes but got {} bytes.",
                    trigger_id_size, number_of_bytes);
                }
            }
            Err(StreamingSocketPairReceiveError::Interrupt) => {
                fail!(from self, with ListenerWaitError::InterruptSignal,
                    "{msg} since an interrupt signal was received.");
            }
            Err(e) => {
                fail!(from self, with ListenerWaitError::InternalFailure,
                    "{msg} due to an internal failure while receiving data on the underlying streaming socket ({:?}).", e);
            }
        }
    }

    fn wait_all_impl<
        WaitCall: FnMut(&mut [u8]) -> Result<usize, StreamingSocketPairReceiveError>,
        F: FnMut(TriggerId),
    >(
        &self,
        mut callback: F,
        waitcall: WaitCall,
        msg: &str,
    ) -> Result<(), ListenerWaitError> {
        match self.wait_one_impl(waitcall, msg)? {
            None => return Ok(()),
            Some(trigger_id) => callback(trigger_id),
        }

        for _ in 0..MAX_BATCH_SIZE {
            match self.wait_one_impl(|buffer| self.socket.try_receive(buffer), msg)? {
                None => return Ok(()),
                Some(trigger_id) => callback(trigger_id),
            }
        }

        Ok(())
    }
}

impl crate::event::Listener for Listener {
    const IS_FILE_DESCRIPTOR_BASED: bool = true;

    fn try_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        self.wait_one_impl(
            |buffer| self.socket.try_receive(buffer),
            "Unable to try to receive a TriggerId",
        )
    }

    fn timed_wait_one(
        &self,
        timeout: core::time::Duration,
    ) -> Result<Option<TriggerId>, ListenerWaitError> {
        self.wait_one_impl(
            |buffer| self.socket.timed_receive(buffer, timeout),
            "Unable to receive a TriggerId with a timeout",
        )
    }

    fn blocking_wait_one(&self) -> Result<Option<TriggerId>, ListenerWaitError> {
        self.wait_one_impl(
            |buffer| self.socket.blocking_receive(buffer),
            "Unable to block until a TriggerId is received",
        )
    }

    fn try_wait_all<F: FnMut(TriggerId)>(&self, callback: F) -> Result<(), ListenerWaitError> {
        self.wait_all_impl(
            callback,
            |buffer| self.socket.try_receive(buffer),
            "Unable to try to receive all TriggerIds",
        )
    }

    fn timed_wait_all<F: FnMut(TriggerId)>(
        &self,
        callback: F,
        timeout: Duration,
    ) -> Result<(), ListenerWaitError> {
        self.wait_all_impl(
            callback,
            |buffer| self.socket.timed_receive(buffer, timeout),
            "Unable to receive all TriggerIds with a timeout",
        )
    }

    fn blocking_wait_all<F: FnMut(TriggerId)>(&self, callback: F) -> Result<(), ListenerWaitError> {
        self.wait_all_impl(
            callback,
            |buffer| self.socket.blocking_receive(buffer),
            "Unable to block until all TriggerIds are received",
        )
    }
}

#[derive(Debug)]
pub struct ListenerBuilder {
    name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<EventImpl> for ListenerBuilder {
    fn new(name: &FileName) -> Self {
        Self {
            name: name.clone(),
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &<EventImpl as super::NamedConceptMgmt>::Configuration) -> Self {
        self.config = config.clone();
        self
    }
}

impl crate::event::ListenerBuilder<EventImpl> for ListenerBuilder {
    fn trigger_id_max(self, _id: TriggerId) -> Self {
        self
    }

    fn create(self) -> Result<Listener, ListenerCreateError> {
        let msg = "Failed to create Listener";
        let full_path = self.config.path_for(&self.name);

        let mut guard = fail!(from self, when PROCESS_LOCAL_STORAGE.lock(),
            with ListenerCreateError::InternalFailure,
            "{msg} due to a failure while acquiring the lock.");
        let entry = guard.get_mut(&full_path);
        if entry.is_some() {
            fail!(from self, with ListenerCreateError::AlreadyExists,
                "{msg} since the event already exists.");
        }

        let (notifier, listener) = match StreamingSocket::create_pair() {
            Ok((notifier, listener)) => (notifier, listener),
            Err(StreamingSocketPairCreationError::InsufficientPermissions) => {
                fail!(from self, with ListenerCreateError::InsufficientPermissions,
                    "{msg} due to insufficient permissions to create a socket pair.");
            }
            Err(e) => {
                fail!(from self, with ListenerCreateError::InternalFailure,
                    "{msg} due to an internal error while creating the socket pair ({:?}).", e);
            }
        };

        guard.insert(full_path, StorageEntry { notifier });

        Ok(Listener {
            name: self.name,
            socket: listener,
            config: self.config,
        })
    }
}
