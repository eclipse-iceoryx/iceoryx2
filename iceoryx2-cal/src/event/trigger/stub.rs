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
    ListenerCreateError, ListenerWaitError, NotifierNotifyError, NotifierOpenError,
    trigger::{Configuration, HandlerInterface},
};
use crate::{
    dynamic_storage::DynamicStorage,
    event::{
        event_state::EventState,
        trigger::{State, WaiterInterface},
    },
};
use core::fmt::Debug;
use core::ptr::NonNull;
use core::{mem::MaybeUninit, time::Duration};
use iceoryx2_bb_elementary_traits::{
    testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_system_types::file_name::FileName;

#[derive(Debug)]
pub(crate) struct Stub {}

unsafe impl Send for Stub {}
unsafe impl Sync for Stub {}
impl Abandonable for Stub {
    unsafe fn abandon_in_place(_this: NonNull<Self>) {
        unimplemented!()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> WaiterInterface<E, Mgmt, Storage> for Stub
{
    fn create(
        _name: &FileName,
        _config: &Configuration,
        _mgmt: &mut MaybeUninit<Mgmt>,
    ) -> Result<Self, ListenerCreateError> {
        unimplemented!()
    }

    fn try_wait(&self) -> Result<(), ListenerWaitError> {
        unimplemented!()
    }

    fn timed_wait(&self, _timeout: Duration) -> Result<(), ListenerWaitError> {
        unimplemented!()
    }

    fn blocking_wait(&self) -> Result<(), ListenerWaitError> {
        unimplemented!()
    }
}
impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> HandlerInterface<E, Mgmt, Storage> for Stub
{
    fn notify(&self) -> Result<(), NotifierNotifyError> {
        unimplemented!()
    }
    fn open(
        _name: &FileName,
        _config: &Configuration,
        _mgmt: &Mgmt,
    ) -> Result<Self, NotifierOpenError> {
        unimplemented!()
    }
}
