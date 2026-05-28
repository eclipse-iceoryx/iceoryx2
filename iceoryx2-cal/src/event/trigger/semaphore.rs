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

use core::fmt::Debug;
use core::marker::PhantomData;

use crate::dynamic_storage::{self, DynamicStorageBuilder};
use crate::event::event_state::{EventActivation, EventId};
use crate::event::trigger::{Trigger, TriggerNotifyError, TriggerWaitError};
use crate::{
    dynamic_storage::DynamicStorage,
    event::{
        NamedConcept, NamedConceptBuilder, NamedConceptMgmt,
        event_state::EventState,
        trigger::{TriggerHandle, TriggerHandleBuilder, TriggerWaiter, TriggerWaiterBuilder},
    },
    named_concept::NamedConceptConfiguration,
};
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use core::time::Duration;
use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::non_null::NonNullCompat;
use iceoryx2_bb_elementary_traits::{
    testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::mutex::IpcCapable;
use iceoryx2_bb_posix::semaphore::{
    SemaphoreInterface, UnnamedSemaphore, UnnamedSemaphoreBuilder, UnnamedSemaphoreHandle,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::fail;

pub(crate) mod internal {
    use super::*;

    pub trait WaiterImpl<
        E: EventState,
        Mgmt: ZeroCopySend + Send + Sync + Debug,
        Storage: DynamicStorage<State<E, Mgmt>>,
    >: Send + Sync + Debug + Abandonable
    {
        fn create(config: &Configuration<E, Mgmt, Storage>, mgmt: &mut MaybeUninit<Mgmt>) -> Self;
        fn try_wait(&self) -> Result<(), TriggerWaitError>;
        fn timed_wait(&self, timeout: Duration) -> Result<(), TriggerWaitError>;
        fn blocking_wait(&self) -> Result<(), TriggerWaitError>;
    }

    pub trait HandlerImpl<
        E: EventState,
        Mgmt: ZeroCopySend + Send + Sync + Debug,
        Storage: DynamicStorage<State<E, Mgmt>>,
    >: Send + Sync + Debug + Abandonable
    {
        fn open(config: &Configuration<E, Mgmt, Storage>, mgmt: &Mgmt) -> Self;
        fn notify(&self) -> Result<(), TriggerNotifyError>;
    }

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
    > WaiterImpl<E, Mgmt, Storage> for Stub
    {
        fn create(
            _config: &Configuration<E, Mgmt, Storage>,
            _mgmt: &mut MaybeUninit<Mgmt>,
        ) -> Self {
            unimplemented!()
        }

        fn try_wait(&self) -> Result<(), TriggerWaitError> {
            unimplemented!()
        }

        fn timed_wait(&self, _timeout: Duration) -> Result<(), TriggerWaitError> {
            unimplemented!()
        }

        fn blocking_wait(&self) -> Result<(), TriggerWaitError> {
            unimplemented!()
        }
    }
    impl<
        E: EventState,
        Mgmt: ZeroCopySend + Send + Sync + Debug,
        Storage: DynamicStorage<State<E, Mgmt>>,
    > HandlerImpl<E, Mgmt, Storage> for Stub
    {
        fn notify(&self) -> Result<(), TriggerNotifyError> {
            unimplemented!()
        }
        fn open(_config: &Configuration<E, Mgmt, Storage>, _mgmt: &Mgmt) -> Self {
            unimplemented!()
        }
    }
}

#[derive(ZeroCopySend)]
#[repr(C)]
pub struct State<E: EventState, Mgmt: ZeroCopySend + Send + Sync + Debug> {
    event: E,
    handle: MaybeUninit<Mgmt>,
    event_id_max: EventId,
    notification_count: AtomicU64,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Configuration<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> {
    value: Storage::Configuration,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> Clone for Configuration<E, Mgmt, Storage>
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _data_1: PhantomData,
            _data_2: PhantomData,
        }
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> Default for Configuration<E, Mgmt, Storage>
{
    fn default() -> Self {
        Self {
            value: Storage::Configuration::default()
                .path_hint(&TriggerImpl::<
                    E,
                    Mgmt,
                    Storage,
                    internal::Stub,
                    internal::Stub,
                >::default_path_hint())
                .suffix(&TriggerImpl::<
                    E,
                    Mgmt,
                    Storage,
                    internal::Stub,
                    internal::Stub,
                >::default_suffix())
                .prefix(&TriggerImpl::<
                    E,
                    Mgmt,
                    Storage,
                    internal::Stub,
                    internal::Stub,
                >::default_prefix()),
            _data_1: PhantomData,
            _data_2: PhantomData,
        }
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> Configuration<E, Mgmt, Storage>
{
    fn to_storage_config(&self) -> Storage::Configuration {
        let mut suffix = *self.get_suffix();
        suffix.push_bytes(b"_mgmt").unwrap();
        self.value.clone().suffix(&suffix)
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> NamedConceptConfiguration for Configuration<E, Mgmt, Storage>
{
    fn prefix(mut self, value: &FileName) -> Self {
        self.value = self.value.prefix(value);
        self
    }

    fn get_prefix(&self) -> &FileName {
        self.value.get_prefix()
    }

    fn suffix(mut self, value: &FileName) -> Self {
        self.value = self.value.suffix(value);
        self
    }

    fn get_suffix(&self) -> &FileName {
        self.value.get_suffix()
    }

    fn path_hint(mut self, value: &Path) -> Self {
        self.value = self.value.path_hint(value);
        self
    }

    fn get_path_hint(&self) -> &Path {
        self.value.get_path_hint()
    }
}

#[derive(Debug)]
pub struct TriggerImpl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> {
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
    _data_3: PhantomData<Storage>,
    _data_4: PhantomData<H>,
    _data_5: PhantomData<W>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> NamedConceptMgmt for TriggerImpl<E, Mgmt, Storage, H, W>
{
    type Configuration = Configuration<E, Mgmt, Storage>;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        Storage::does_exist_cfg(name, &cfg.value)
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        Storage::list_cfg(&cfg.value)
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        unsafe { Storage::remove_cfg(name, &cfg.value) }
    }

    fn remove_path_hint(
        value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Storage::remove_path_hint(value)
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> super::Trigger<E> for TriggerImpl<E, Mgmt, Storage, H, W>
{
    type Handle = Handle<E, Mgmt, Storage, H>;
    type HandleBuilder = HandleBuilder<E, Mgmt, Storage, H, W>;
    type Waiter = Waiter<E, Mgmt, Storage, W>;
    type WaiterBuiler = WaiterBuilder<E, Mgmt, Storage, H, W>;
}

#[derive(Debug)]
pub struct Handle<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
> {
    storage: Storage,
    handle: H,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
> Abandonable for Handle<E, Mgmt, Storage, H>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { H::abandon_in_place(NonNull::iox2_from_mut(&mut this.handle)) };
        unsafe { Storage::abandon_in_place(NonNull::iox2_from_mut(&mut this.storage)) };
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
> NamedConcept for Handle<E, Mgmt, Storage, H>
{
    fn name(&self) -> &FileName {
        self.storage.name()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
> TriggerHandle<E> for Handle<E, Mgmt, Storage, H>
{
    fn event_id_max(&self) -> EventId {
        self.storage.get().event_id_max
    }

    fn notify(&self, event_id: EventId) -> Result<(), super::TriggerNotifyError> {
        let msg = "Unable to notify";
        fail!(from self,
              when self.storage.get().event.activate(event_id),
              "{msg} with {event_id:?} since the activation failed.");
        fail!(from self,
              when self.handle.notify(),
              "{msg} with {event_id:?} since the notification could not be sent.");
        Ok(())
    }
}

#[derive(Debug)]
pub struct Waiter<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> {
    storage: Storage,
    waiter: W,
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> Abandonable for Waiter<E, Mgmt, Storage, W>
{
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { W::abandon_in_place(NonNull::iox2_from_mut(&mut this.waiter)) };
        unsafe { Storage::abandon_in_place(NonNull::iox2_from_mut(&mut this.storage)) };
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> NamedConcept for Waiter<E, Mgmt, Storage, W>
{
    fn name(&self) -> &FileName {
        self.storage.name()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> TriggerWaiter<E> for Waiter<E, Mgmt, Storage, W>
{
    fn try_wait<F: FnMut(EventActivation)>(&self, callback: F) -> Result<u64, TriggerWaitError> {
        let msg = "Failed to try wait and acquire all event notifications";
        fail!(from self,
              when self.waiter.try_wait(),
              "{msg} since the underlying waiting call failed.");

        let number_of_events = self.storage.get().event.drain(callback);
        Ok(number_of_events)
    }

    fn timed_wait<F: FnMut(EventActivation)>(
        &self,
        callback: F,
        timeout: Duration,
    ) -> Result<u64, TriggerWaitError> {
        let msg = "Failed to wait with timeout and acquire all event notifications";
        fail!(from self,
              when self.waiter.timed_wait(timeout),
              "{msg} since the underlying waiting call failed.");

        let number_of_events = self.storage.get().event.drain(callback);
        Ok(number_of_events)
    }

    fn blocking_wait<F: FnMut(EventActivation)>(
        &self,
        callback: F,
    ) -> Result<u64, TriggerWaitError> {
        let msg = "Failed to wait with timeout and acquire all event notifications";
        fail!(from self,
              when self.waiter.blocking_wait(),
              "{msg} since the underlying waiting call failed.");

        let number_of_events = self.storage.get().event.drain(callback);
        Ok(number_of_events)
    }
}

#[derive(Debug)]
pub struct WaiterBuilder<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> {
    name: FileName,
    config: Configuration<E, Mgmt, Storage>,
    timeout: Duration,
    _data_1: PhantomData<H>,
    _data_2: PhantomData<W>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> NamedConceptBuilder<TriggerImpl<E, Mgmt, Storage, H, W>>
    for WaiterBuilder<E, Mgmt, Storage, H, W>
{
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            timeout: Duration::ZERO,
            config: Configuration::default(),
            _data_1: PhantomData,
            _data_2: PhantomData,
        }
    }

    fn config(
        mut self,
        config: &<TriggerImpl<E, Mgmt, Storage, H, W> as NamedConceptMgmt>::Configuration,
    ) -> Self {
        self.config = config.clone();
        self
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> TriggerWaiterBuilder<E, TriggerImpl<E, Mgmt, Storage, H, W>>
    for WaiterBuilder<E, Mgmt, Storage, H, W>
{
    fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn open(
        self,
    ) -> Result<
        <TriggerImpl<E, Mgmt, Storage, H, W> as super::Trigger<E>>::Handle,
        super::TriggerOpenError,
    > {
        let storage = Storage::Builder::new(&self.name)
            .config(&self.config.to_storage_config())
            .has_ownership(true)
            .open(AccessMode::ReadWrite)
            .unwrap();

        let handle = H::open(&self.config, unsafe {
            storage.get().handle.assume_init_ref()
        });

        Ok(Handle {
            storage,
            handle,
            _data_1: PhantomData,
            _data_2: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct HandleBuilder<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> {
    name: FileName,
    event_id_max: EventId,
    config: Configuration<E, Mgmt, Storage>,
    _data_1: PhantomData<H>,
    _data_2: PhantomData<W>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> NamedConceptBuilder<TriggerImpl<E, Mgmt, Storage, H, W>>
    for HandleBuilder<E, Mgmt, Storage, H, W>
{
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            event_id_max: EventId::new(8),
            config: Configuration::default(),
            _data_1: PhantomData,
            _data_2: PhantomData,
        }
    }

    fn config(
        mut self,
        config: &<TriggerImpl<E, Mgmt, Storage, H, W> as NamedConceptMgmt>::Configuration,
    ) -> Self {
        self.config = config.clone();
        self
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: internal::HandlerImpl<E, Mgmt, Storage>,
    W: internal::WaiterImpl<E, Mgmt, Storage>,
> TriggerHandleBuilder<E, TriggerImpl<E, Mgmt, Storage, H, W>>
    for HandleBuilder<E, Mgmt, Storage, H, W>
{
    fn event_id_max(mut self, id: EventId) -> Self {
        self.event_id_max = id;
        self
    }

    fn create(
        self,
    ) -> Result<
        <TriggerImpl<E, Mgmt, Storage, H, W> as super::Trigger<E>>::Waiter,
        super::TriggerCreateError,
    > {
        let state_size = E::memory_size((self.event_id_max.as_value() + 1) as usize);
        let mut waiter = None;
        let storage = Storage::Builder::new(&self.name)
            .config(&self.config.to_storage_config())
            .has_ownership(true)
            .supplementary_size(state_size)
            .initializer(|value, allocator| {
                value.write(State {
                    event: unsafe { E::new_uninit((self.event_id_max.as_value() + 1) as usize) },
                    handle: MaybeUninit::uninit(),
                    event_id_max: self.event_id_max,
                    notification_count: AtomicU64::new(0),
                });

                unsafe { value.assume_init_mut().event.init(allocator).unwrap() };
                waiter = Some(W::create(
                    &self.config,
                    &mut unsafe { value.assume_init_mut() }.handle,
                ));

                true
            })
            .create()
            .unwrap();

        Ok(Waiter {
            storage,
            waiter: waiter.unwrap(),
            _data_1: PhantomData,
            _data_2: PhantomData,
        })
    }
}

/////////// semaphore impl

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
    internal::HandlerImpl<E, SemaphoreMgmt, Storage> for SemaphoreHandle<E, Storage>
{
    fn open(_config: &Configuration<E, SemaphoreMgmt, Storage>, mgmt: &SemaphoreMgmt) -> Self {
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

    fn notify(&self) -> Result<(), TriggerNotifyError> {
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
    internal::WaiterImpl<E, SemaphoreMgmt, Storage> for SemaphoreWaiter<E, Storage>
{
    fn create(
        _config: &Configuration<E, SemaphoreMgmt, Storage>,
        mgmt: &mut MaybeUninit<SemaphoreMgmt>,
    ) -> Self {
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

    fn try_wait(&self) -> Result<(), TriggerWaitError> {
        self.semaphore.try_wait().unwrap();
        Ok(())
    }

    fn timed_wait(&self, timeout: Duration) -> Result<(), TriggerWaitError> {
        self.semaphore.timed_wait(timeout).unwrap();
        Ok(())
    }

    fn blocking_wait(&self) -> Result<(), TriggerWaitError> {
        self.semaphore.blocking_wait().unwrap();
        Ok(())
    }
}

#[allow(type_alias_bounds)] // they are not enforced, but we keep them to communicate the contract
pub type GenericSemaphoreTrigger<E: EventState, Storage: DynamicStorage<State<E, SemaphoreMgmt>>> =
    TriggerImpl<E, SemaphoreMgmt, Storage, SemaphoreHandle<E, Storage>, SemaphoreWaiter<E, Storage>>;

pub type SemaphoreShmBitSet = GenericSemaphoreTrigger<
    RelocatableBitSet,
    dynamic_storage::posix_shared_memory::Storage<State<RelocatableBitSet, SemaphoreMgmt>>,
>;
