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

use crate::{
    dynamic_storage::{DynamicStorage, DynamicStorageBuilder},
    event::{
        Event, EventId, Listener, ListenerBuilder, ListenerCreateError, ListenerWaitError,
        NamedConcept, NamedConceptBuilder, NamedConceptMgmt, Notifier, NotifierBuilder,
        NotifierOpenError,
        event_state::{EventActivation, EventState},
        trigger::{HandlerInterface, State, WaiterInterface, stub::Stub},
    },
    named_concept::NamedConceptConfiguration,
};
use core::fmt::Debug;
use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull, time::Duration};
use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary_traits::{
    non_null::NonNullCompat, testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::fail;

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
                .path_hint(&EventImpl::<E, Mgmt, Storage, Stub, Stub>::default_path_hint())
                .suffix(&EventImpl::<E, Mgmt, Storage, Stub, Stub>::default_suffix())
                .prefix(&EventImpl::<E, Mgmt, Storage, Stub, Stub>::default_prefix()),
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

    fn to_trigger_config(&self) -> super::trigger::Configuration {
        super::trigger::Configuration {
            suffix: *self.get_suffix(),
            prefix: *self.get_prefix(),
            path_hint: *self.get_path_hint(),
        }
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
pub struct EventImpl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
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
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
> NamedConceptMgmt for EventImpl<E, Mgmt, Storage, H, W>
{
    type Configuration = Configuration<E, Mgmt, Storage>;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        Storage::does_exist_cfg(name, &cfg.to_storage_config())
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        Storage::list_cfg(&cfg.to_storage_config())
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        unsafe { Storage::remove_cfg(name, &cfg.to_storage_config()) }
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
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
> Event<E> for EventImpl<E, Mgmt, Storage, H, W>
{
    type Listener = Waiter<E, Mgmt, Storage, W>;
    type ListenerBuilder = WaiterBuilder<E, Mgmt, Storage, H, W>;
    type Notifier = Handle<E, Mgmt, Storage, H>;
    type NotifierBuilder = HandleBuilder<E, Mgmt, Storage, H, W>;
}

#[derive(Debug)]
pub struct Handle<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: HandlerInterface<E, Mgmt, Storage>,
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
    H: HandlerInterface<E, Mgmt, Storage>,
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
    H: HandlerInterface<E, Mgmt, Storage>,
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
    H: HandlerInterface<E, Mgmt, Storage>,
> Notifier<E> for Handle<E, Mgmt, Storage, H>
{
    fn event_id_max(&self) -> EventId {
        self.storage.get().event_id_max
    }

    fn notify(&self, event_id: EventId) -> Result<(), super::NotifierNotifyError> {
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
    W: WaiterInterface<E, Mgmt, Storage>,
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
    W: WaiterInterface<E, Mgmt, Storage>,
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
    W: WaiterInterface<E, Mgmt, Storage>,
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
    W: WaiterInterface<E, Mgmt, Storage>,
> Listener<E> for Waiter<E, Mgmt, Storage, W>
{
    fn try_wait<F: FnMut(EventActivation)>(&self, callback: F) -> Result<u64, ListenerWaitError> {
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
    ) -> Result<u64, ListenerWaitError> {
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
    ) -> Result<u64, ListenerWaitError> {
        let msg = "Failed to wait with timeout and acquire all event notifications";
        fail!(from self,
              when self.waiter.blocking_wait(),
              "{msg} since the underlying waiting call failed.");

        let number_of_events = self.storage.get().event.drain(callback);
        Ok(number_of_events)
    }
}

#[derive(Debug)]
pub struct HandleBuilder<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
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
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
> NamedConceptBuilder<EventImpl<E, Mgmt, Storage, H, W>> for HandleBuilder<E, Mgmt, Storage, H, W>
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
        config: &<EventImpl<E, Mgmt, Storage, H, W> as NamedConceptMgmt>::Configuration,
    ) -> Self {
        self.config = config.clone();
        self
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
> NotifierBuilder<E, EventImpl<E, Mgmt, Storage, H, W>> for HandleBuilder<E, Mgmt, Storage, H, W>
{
    fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn open(
        self,
    ) -> Result<<EventImpl<E, Mgmt, Storage, H, W> as Event<E>>::Notifier, NotifierOpenError> {
        let storage = Storage::Builder::new(&self.name)
            .config(&self.config.to_storage_config())
            .has_ownership(false)
            .open(AccessMode::ReadWrite)
            .unwrap();

        let handle = H::open(&self.config.to_trigger_config(), unsafe {
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
pub struct WaiterBuilder<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
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
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
> NamedConceptBuilder<EventImpl<E, Mgmt, Storage, H, W>> for WaiterBuilder<E, Mgmt, Storage, H, W>
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
        config: &<EventImpl<E, Mgmt, Storage, H, W> as NamedConceptMgmt>::Configuration,
    ) -> Self {
        self.config = config.clone();
        self
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
    H: HandlerInterface<E, Mgmt, Storage>,
    W: WaiterInterface<E, Mgmt, Storage>,
> ListenerBuilder<E, EventImpl<E, Mgmt, Storage, H, W>> for WaiterBuilder<E, Mgmt, Storage, H, W>
{
    fn event_id_max(mut self, id: EventId) -> Self {
        self.event_id_max = id;
        self
    }

    fn create(
        self,
    ) -> Result<<EventImpl<E, Mgmt, Storage, H, W> as Event<E>>::Listener, ListenerCreateError>
    {
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
                    &self.config.to_trigger_config(),
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
