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

use super::Configuration;
use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull, time::Duration};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::{
    testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;
use iceoryx2_bb_posix::{
    mutex::IpcCapable,
    semaphore::{
        SemaphoreInterface, UnnamedSemaphore, UnnamedSemaphoreBuilder, UnnamedSemaphoreHandle,
    },
};

use crate::{
    dynamic_storage::{self, DynamicStorage},
    event::{
        ListenerWaitError, NotifierNotifyError,
        common::EventImpl,
        event_state::EventState,
        trigger::{HandlerInterface, State, WaiterInterface},
    },
};

#[derive(Debug, ZeroCopySend)]
#[repr(C)]
pub struct SemaphoreMgmt {
    handle: UnnamedSemaphoreHandle,
}

#[derive(Debug)]
pub struct SemaphoreHandle<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> {
    semaphore: UnnamedSemaphore<'static>,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

unsafe impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> Send
    for SemaphoreHandle<E, Storage>
{
}
unsafe impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> Sync
    for SemaphoreHandle<E, Storage>
{
}

impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> Abandonable
    for SemaphoreHandle<E, Storage>
{
    unsafe fn abandon_in_place(_this: NonNull<Self>) {}
}

impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>>
    HandlerInterface<E, SemaphoreMgmt, Storage> for SemaphoreHandle<E, Storage>
{
    fn open(_config: &Configuration, mgmt: &SemaphoreMgmt) -> Self {
        Self {
            semaphore: unsafe {
                UnnamedSemaphore::from_ipc_handle(core::mem::transmute::<
                    &UnnamedSemaphoreHandle,
                    &'static UnnamedSemaphoreHandle,
                >(&mgmt.handle))
            },
            _data_1: PhantomData,
            _data_2: PhantomData,
        }
    }

    fn notify(&self) -> Result<(), NotifierNotifyError> {
        self.semaphore.post().unwrap();
        Ok(())
    }
}

#[derive(Debug)]
pub struct SemaphoreWaiter<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> {
    semaphore_mgmt: *mut SemaphoreMgmt,
    semaphore: UnnamedSemaphore<'static>,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Storage>,
}

unsafe impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> Send
    for SemaphoreWaiter<E, Storage>
{
}
unsafe impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> Sync
    for SemaphoreWaiter<E, Storage>
{
}

impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> Abandonable
    for SemaphoreWaiter<E, Storage>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { core::ptr::drop_in_place(this.semaphore_mgmt) };
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> Drop
    for SemaphoreWaiter<E, Storage>
{
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.semaphore_mgmt) };
    }
}

impl<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>>
    WaiterInterface<E, SemaphoreMgmt, Storage> for SemaphoreWaiter<E, Storage>
{
    fn create(_config: &Configuration, mgmt: &mut MaybeUninit<SemaphoreMgmt>) -> Self {
        use iceoryx2_bb_posix::ipc_capable::Handle;

        mgmt.write(SemaphoreMgmt {
            handle: UnnamedSemaphoreHandle::new(),
        });

        Self {
            semaphore_mgmt: mgmt.as_mut_ptr(),
            semaphore: UnnamedSemaphoreBuilder::new()
                .initial_value(0)
                .is_interprocess_capable(true)
                .create(&unsafe { &*mgmt.as_ptr() }.handle)
                .unwrap(),
            _data_1: PhantomData,
            _data_2: PhantomData,
        }
    }

    fn try_wait(&self) -> Result<(), ListenerWaitError> {
        self.semaphore.try_wait().unwrap();
        Ok(())
    }

    fn timed_wait(&self, timeout: Duration) -> Result<(), ListenerWaitError> {
        self.semaphore.timed_wait(timeout).unwrap();
        Ok(())
    }

    fn blocking_wait(&self) -> Result<(), ListenerWaitError> {
        self.semaphore.blocking_wait().unwrap();
        Ok(())
    }
}

#[allow(type_alias_bounds)] // they are not enforced, but we keep them to communicate the contract
pub type GenericSemaphoreTrigger<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> =
    EventImpl<E, SemaphoreMgmt, Storage, SemaphoreHandle<E, Storage>, SemaphoreWaiter<E, Storage>>;

pub type SemaphoreShmBitSet = GenericSemaphoreTrigger<
    RelocatableBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableBitSet, SemaphoreMgmt>>,
>;
