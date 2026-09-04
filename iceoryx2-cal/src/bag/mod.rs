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

//! Offers an interface to a container to concurrently add, remove and access
//! values with a fixed position for the lifetime of the data. The data in the
//! container is not necessarily ordered.
//!
//! # Example
//!
//! ```
//! use core::ptr::NonNull;
//!
//! use iceoryx2_bb_container::queue::RelocatableContainer;
//! use iceoryx2_bb_elementary_traits::allocator::{Allocate, AllocationError};
//! use iceoryx2_cal::bag::*;
//!
//! fn create_bag<B: BagFamily, T: BagType, A: Allocate<NonNull<u8>>>(
//!     capacity: usize,
//!     allocator: A,
//! ) -> Result<B::Bag<T>, AllocationError> {
//!     let mut bag = unsafe { B::Bag::<T>::new_uninit(capacity) };
//!     unsafe {
//!         bag.init(&allocator)?;
//!     }
//!     Ok(bag)
//! }
//! ```

pub mod default_bag;
pub mod recommended;

use core::fmt::Debug;

use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use iceoryx2_bb_lock_free::mpmc::container::{
    ContainerAddFailure, ContainerHandle, ContainerRemoveError, ContainerState,
};
use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::OwnerId;
use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::{ReleaseMode, ReleaseState};

pub type BagHandle = ContainerHandle;
pub type BagState<T> = ContainerState<T>;
pub type BagAddFailure = ContainerAddFailure;
pub type BagRemoveError = ContainerRemoveError;

/// A super trait defining the trait bounds of a [`Bag`] type
pub trait BagType: Copy + Debug + ZeroCopySend {}
impl<T: Copy + Debug + ZeroCopySend> BagType for T {}

/// The [`BagFamily`] provides the associated type for the concrete type implementing the concept
pub trait BagFamily: Debug + 'static {
    type Bag<T: BagType>: Debug + Send + Sync + ZeroCopySend + RelocatableContainer + Bag<T>;
}

/// The [`Bag`] trait provides access to an unordered container with fix position for the data
/// during its lifetime
pub trait Bag<T: BagType>: Debug {
    /// Returns the capacity of the bag.
    fn capacity(&self) -> usize;

    /// Returns the current len of the bag
    fn len(&self) -> usize;

    /// Returns true if the container is empty, otherwise false
    fn is_empty(&self) -> bool;

    /// Adds a new element to the [`Bag`]. If there is no more space available it returns
    /// [`None`], otherwise [`Some`] containing the the index value to the underlying element.
    ///
    /// Must be released with [`Bag::remove()`].
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Bag::init()`](RelocatableContainer::init()) was called before calling this method
    ///
    unsafe fn add(
        &self,
        value: T,
        owner_id: OwnerId,
    ) -> Result<(*const T, BagHandle), BagAddFailure>
    where
        T: PartialEq;

    /// Useful in IPC context when an application holding the UniqueIndex has died.
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Bag::init()`](RelocatableContainer::init()) was called before calling this method
    ///  * Ensure that no one else possesses the [`BagHandle`] and the index was unrecoverable
    ///    lost
    ///  * Ensure that the `handle` was acquired by the same [`Bag`]
    ///    with [`Bag::add()`], otherwise the method will panic.
    ///
    /// **Important:** If the [`BagHandle`] still exists it causes double frees or freeing an index
    /// which was allocated afterwards
    ///
    unsafe fn remove(
        &self,
        handle: BagHandle,
        mode: ReleaseMode,
    ) -> Result<ReleaseState, BagRemoveError>;

    /// Returns [`BagState`] which contains all elements of this bag. Be aware that
    /// this state can be out of date as soon as it is returned from this function.
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Bag::init()`](RelocatableContainer::init()) was called before calling this method
    ///
    unsafe fn get_state(&self) -> BagState<T>;

    /// Recovers and releases all entries the dead [`OwnerId`] owned. It assumes that the dead owner
    /// maybe died while adding some entry, therefore it removes all entries where the
    /// [`OwnerId`] does not contain any data or where there was data and the provided predicate
    /// returned [`true`].
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Bag::init()`](RelocatableContainer::init()) was called before calling this method
    ///  * All existing [`BagHandle`] that belong to the [`OwnerId`] must never be removed with
    ///    [`Bag::remove()`] otherwise we corrupt the state.
    ///
    unsafe fn recover<F: FnMut(T) -> bool>(
        &self,
        dead_owner_id: OwnerId,
        predicate: F,
        mode: ReleaseMode,
    ) -> ReleaseState;

    /// Syncs the [`BagState`] with the current state of the [`Bag`]. If the state has
    /// changed it returns true, otherwise false.
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Bag::init()`](RelocatableContainer::init()) was called before calling this method
    ///  * Ensure that the input argument `previous_state` was acquired by the same [`Bag`]
    ///    with [`Bag::get_state()`], otherwise the method will panic.
    ///
    unsafe fn update_state(&self, previous_state: &mut BagState<T>) -> bool;
}
