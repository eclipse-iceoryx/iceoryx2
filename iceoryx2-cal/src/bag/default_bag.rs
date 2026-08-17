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

use crate::bag::{Bag, BagAddFailure, BagFamily, BagHandle, BagRemoveError, BagState, BagType};

use iceoryx2_bb_lock_free::mpmc::container::Container;
use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::OwnerId;
use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::{ReleaseMode, ReleaseState};

#[derive(Debug)]
pub struct DefaultBag;

impl BagFamily for DefaultBag {
    type Bag<T: BagType> = Container<T>;
}

impl<T: BagType> Bag<T> for Container<T> {
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
    ) -> Result<(*const T, BagHandle), BagAddFailure> {
        unsafe { self.add(value, owner_id) }
    }

    unsafe fn remove(
        &self,
        handle: BagHandle,
        mode: ReleaseMode,
    ) -> Result<ReleaseState, BagRemoveError> {
        unsafe { self.remove(handle, mode) }
    }

    unsafe fn get_state(&self) -> BagState<T> {
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

    unsafe fn update_state(&self, previous_state: &mut BagState<T>) -> bool {
        unsafe { self.update_state(previous_state) }
    }
}
