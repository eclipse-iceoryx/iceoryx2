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
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::{
    testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;

mod internal {
    use super::*;

    trait WaiterImpl<
        E: EventState,
        Mgmt: ZeroCopySend + Send + Sync + Debug,
        Storage: DynamicStorage<State<E, Mgmt>>,
    >
    {
        fn create(config: &Configuration<E, Mgmt, Storage>, mgmt: &mut MaybeUninit<Mgmt>) -> Self;
        fn wait(&self) -> Result<(), TriggerWaitError>;
    }

    trait HandlerImpl<
        E: EventState,
        Mgmt: ZeroCopySend + Send + Sync + Debug,
        Storage: DynamicStorage<State<E, Mgmt>>,
    >
    {
        fn open(config: &Configuration<E, Mgmt, Storage>, mgmt: &Mgmt) -> Self;
        fn notify(&self) -> Result<(), TriggerNotifyError>;
    }
}

#[derive(ZeroCopySend)]
#[repr(C)]
pub struct State<E: EventState, Mgmt: ZeroCopySend + Send + Sync + Debug> {
    event: E,
    handle: Mgmt,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Configuration<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> {
    value: Storage::Configuration,
    _data1: PhantomData<E>,
    _data2: PhantomData<Mgmt>,
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
            _data1: PhantomData,
            _data2: PhantomData,
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
                .path_hint(&TriggerImpl::<E, Mgmt, Storage>::default_path_hint())
                .suffix(&TriggerImpl::<E, Mgmt, Storage>::default_suffix())
                .prefix(&TriggerImpl::<E, Mgmt, Storage>::default_prefix()),
            _data1: PhantomData,
            _data2: PhantomData,
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
pub struct TriggerImpl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> {
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
    _data_3: PhantomData<Storage>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> NamedConceptMgmt for TriggerImpl<E, Mgmt, Storage>
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
> super::Trigger<E> for TriggerImpl<E, Mgmt, Storage>
{
    type Handle = Handle<E, Mgmt, Storage>;
    type HandleBuilder = HandleBuilder<E, Mgmt, Storage>;
    type Waiter = Waiter<E, Mgmt, Storage>;
    type WaiterBuiler = WaiterBuilder<E, Mgmt, Storage>;
}

#[derive(Debug)]
pub struct Handle<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> {
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
    _data_3: PhantomData<Storage>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> Abandonable for Handle<E, Mgmt, Storage>
{
    unsafe fn abandon_in_place(this: std::ptr::NonNull<Self>) {
        todo!()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> NamedConcept for Handle<E, Mgmt, Storage>
{
    fn name(&self) -> &FileName {
        todo!()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> TriggerHandle<E> for Handle<E, Mgmt, Storage>
{
    fn notify(&self) -> Result<(), super::TriggerNotifyError> {
        todo!()
    }

    fn state(&self) -> &E {
        todo!()
    }
}

#[derive(Debug)]
pub struct Waiter<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> {
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
    _data_3: PhantomData<Storage>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> Abandonable for Waiter<E, Mgmt, Storage>
{
    unsafe fn abandon_in_place(this: std::ptr::NonNull<Self>) {
        todo!()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> NamedConcept for Waiter<E, Mgmt, Storage>
{
    fn name(&self) -> &FileName {
        todo!()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> TriggerWaiter<E> for Waiter<E, Mgmt, Storage>
{
    fn state(&self) -> &E {
        todo!()
    }

    fn wait(&self) -> Result<(), super::TriggerWaitError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct WaiterBuilder<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> {
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
    _data_3: PhantomData<Storage>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> NamedConceptBuilder<TriggerImpl<E, Mgmt, Storage>> for WaiterBuilder<E, Mgmt, Storage>
{
    fn new(name: &FileName) -> Self {
        todo!()
    }

    fn config(
        self,
        config: &<TriggerImpl<E, Mgmt, Storage> as NamedConceptMgmt>::Configuration,
    ) -> Self {
        todo!()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> TriggerWaiterBuilder<E, TriggerImpl<E, Mgmt, Storage>> for WaiterBuilder<E, Mgmt, Storage>
{
    fn open(
        self,
    ) -> Result<<TriggerImpl<E, Mgmt, Storage> as super::Trigger<E>>::Handle, super::TriggerOpenError>
    {
        todo!()
    }
}

#[derive(Debug)]
pub struct HandleBuilder<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> {
    _data_1: PhantomData<E>,
    _data_2: PhantomData<Mgmt>,
    _data_3: PhantomData<Storage>,
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> NamedConceptBuilder<TriggerImpl<E, Mgmt, Storage>> for HandleBuilder<E, Mgmt, Storage>
{
    fn new(name: &FileName) -> Self {
        todo!()
    }

    fn config(
        self,
        config: &<TriggerImpl<E, Mgmt, Storage> as NamedConceptMgmt>::Configuration,
    ) -> Self {
        todo!()
    }
}

impl<
    E: EventState,
    Mgmt: ZeroCopySend + Send + Sync + Debug,
    Storage: DynamicStorage<State<E, Mgmt>>,
> TriggerHandleBuilder<E, TriggerImpl<E, Mgmt, Storage>> for HandleBuilder<E, Mgmt, Storage>
{
    fn create(
        self,
    ) -> Result<
        <TriggerImpl<E, Mgmt, Storage> as super::Trigger<E>>::Waiter,
        super::TriggerCreateError,
    > {
        todo!()
    }
}
