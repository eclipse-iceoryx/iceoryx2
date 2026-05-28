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
use crate::{dynamic_storage::DynamicStorage, event::event_state::EventState};
use core::fmt::Debug;
use core::mem::MaybeUninit;
use core::time::Duration;
use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;

pub mod semaphore;
pub mod stub;

pub struct Configuration {
    pub suffix: FileName,
    pub prefix: FileName,
    pub path_hint: Path,
}

#[derive(ZeroCopySend, Debug)]
#[repr(C)]
pub struct State<E: EventState, Mgmt: ZeroCopySend + Send + Sync + Debug> {
    pub event: E,
    pub handle: MaybeUninit<Mgmt>,
    pub event_id_max: EventId,
    pub notification_count: AtomicU64,
}

pub trait WaiterInterface<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
>: Send + Sync + Debug + Abandonable
{
    fn create(
        config: &Configuration,
        mgmt: &mut MaybeUninit<Mgmt>,
    ) -> Result<Self, ListenerCreateError>;
    fn try_wait(&self) -> Result<(), ListenerWaitError>;
    fn timed_wait(&self, timeout: Duration) -> Result<(), ListenerWaitError>;
    fn blocking_wait(&self) -> Result<(), ListenerWaitError>;
}

pub trait HandlerInterface<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
>: Send + Sync + Debug + Abandonable
{
    fn open(config: &Configuration, mgmt: &Mgmt) -> Result<Self, NotifierOpenError>;
    fn notify(&self) -> Result<(), NotifierNotifyError>;
}
