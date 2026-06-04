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

use crate::event::{
    EventId, ListenerCreateError, ListenerWaitError, NotifierNotifyError, NotifierOpenError,
};
use crate::named_concept::NamedConceptRemoveError;
use crate::{dynamic_storage::DynamicStorage, event::event_state::EventState};
use core::fmt::Debug;
use core::mem::MaybeUninit;
use core::time::Duration;
use iceoryx2_bb_concurrency::atomic::AtomicU8;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::fatal_panic;

pub mod semaphore;
pub mod socket_pair;
pub mod stub;
pub mod unix_datagram_socket;

#[derive(Debug)]
pub struct Configuration {
    pub suffix: FileName,
    pub prefix: FileName,
    pub path_hint: Path,
}

impl Configuration {
    pub fn path_for(&self, value: &FileName) -> FilePath {
        let mut path = self.path_hint;
        fatal_panic!(from self, when path.add_path_entry(&self.prefix.into()),
                    "The path hint \"{}\" in combination with the prefix \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, self.prefix, Path::max_len());
        fatal_panic!(from self, when path.push_bytes(value.as_string()),
                    "The path hint \"{}\" in combination with the file name \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, Path::max_len());
        fatal_panic!(from self, when path.push_bytes(self.suffix.as_bytes()),
                    "The path hint \"{}\" in combination with the file name \"{}\" and the suffix \"{}\" exceed the maximum supported path length of {} of the operating system.",
                    path, value, self.suffix, Path::max_len());

        unsafe { FilePath::new_unchecked(path.as_bytes()) }
    }
}

#[derive(ZeroCopySend, Debug)]
#[repr(C)]
pub struct State<E: EventState, Mgmt: ZeroCopySend + Send + Sync + Debug> {
    pub event: E,
    pub handle: MaybeUninit<Mgmt>,
    pub event_id_max: EventId,
    pub notification_state: AtomicU8,
}

pub trait WaiterInterface<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
>: Send + Sync + Debug + Abandonable
{
    const IS_FILE_DESCRIPTOR_BASED: bool;

    /// # Safety
    ///
    ///  * Must ensure that the Waiter or the Handler is currenty not in use by another process.
    ///
    unsafe fn remove(
        name: &FileName,
        config: &Configuration,
    ) -> Result<bool, NamedConceptRemoveError>;
    fn create(
        name: &FileName,
        config: &Configuration,
        mgmt: &mut MaybeUninit<Mgmt>,
    ) -> Result<Self, ListenerCreateError>;
    fn try_wait(&self) -> Result<(), ListenerWaitError>;
    fn timed_wait(&self, timeout: Duration) -> Result<(), ListenerWaitError>;
    fn blocking_wait(&self) -> Result<(), ListenerWaitError>;
    fn empty_buffer(&self) -> Result<(), ListenerWaitError>;
}

pub trait HandlerInterface<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
>: Send + Sync + Debug + Abandonable
{
    fn open(
        name: &FileName,
        config: &Configuration,
        mgmt: &Mgmt,
    ) -> Result<Self, NotifierOpenError>;
    fn notify(&self) -> Result<(), NotifierNotifyError>;
}
