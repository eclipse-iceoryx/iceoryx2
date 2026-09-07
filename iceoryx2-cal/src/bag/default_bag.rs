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

//! The default implementation for the [`BagFamily`] concept and the [`Bag`] trait.

use crate::bag::{
    Bag, BagAddFailure, BagFamily, BagHandleFamily, BagRemoveError, BagStateFamily, BagType,
};

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_lock_free::mpmc::container::{
    Container, ContainerAddFailure, ContainerHandle, ContainerRemoveError, ContainerState,
};
use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::OwnerId;
use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::{ReleaseMode, ReleaseState};

impl From<ContainerAddFailure> for BagAddFailure {
    fn from(value: ContainerAddFailure) -> Self {
        match value {
            ContainerAddFailure::OutOfSpace => BagAddFailure::OutOfSpace,
            ContainerAddFailure::IsLocked => BagAddFailure::IsLocked,
        }
    }
}

impl From<ContainerRemoveError> for BagRemoveError {
    fn from(value: ContainerRemoveError) -> Self {
        match value {
            ContainerRemoveError::ContainerHandleNotOwnedByContainer => {
                BagRemoveError::HandleNotOwnedByInstance
            }
        }
    }
}

impl BagHandleFamily for ContainerHandle {
    fn index(&self) -> usize {
        self.index()
    }
}

impl<T: BagType> BagStateFamily<T> for ContainerState<T> {
    fn for_each<F: FnMut(usize, &T) -> CallbackProgression>(&self, callback: F) {
        self.for_each(callback)
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.get(index)
    }
}

#[derive(Debug)]
pub struct DefaultBag;

impl BagFamily for DefaultBag {
    type BagHandle = ContainerHandle;
    type BagState<T: BagType> = ContainerState<T>;
    type Bag<T: BagType> = Container<T>;
}

impl<T: BagType> Bag<T> for Container<T> {
    type BagHandle = ContainerHandle;
    type BagState<TT: BagType> = ContainerState<T>;

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    unsafe fn add(
        &self,
        value: T,
        owner_id: OwnerId,
    ) -> Result<(*const T, Self::BagHandle), BagAddFailure>
    where
        T: PartialEq,
    {
        unsafe { self.add(value, owner_id).map_err(|e| e.into()) }
    }

    unsafe fn remove(
        &self,
        handle: Self::BagHandle,
        mode: ReleaseMode,
    ) -> Result<ReleaseState, BagRemoveError> {
        unsafe { self.remove(handle, mode).map_err(|e| e.into()) }
    }

    unsafe fn get_state(&self) -> Self::BagState<T> {
        unsafe { self.get_state() }
    }

    unsafe fn recover<F: FnMut(T) -> bool>(
        &self,
        dead_owner_id: OwnerId,
        predicate: F,
        mode: ReleaseMode,
    ) -> ReleaseState {
        unsafe { self.recover(dead_owner_id, predicate, mode) }
    }

    unsafe fn update_state(&self, previous_state: &mut Self::BagState<T>) -> bool {
        unsafe { self.update_state(previous_state) }
    }
}
