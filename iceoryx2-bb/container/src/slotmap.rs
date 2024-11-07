// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use crate::queue::details::MetaQueue;
use crate::vec::details::MetaVec;
use crate::{queue::RelocatableQueue, vec::RelocatableVec};
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::math::align_to;
use iceoryx2_bb_elementary::owning_pointer::OwningPointer;
use iceoryx2_bb_elementary::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_log::fail;
use std::mem::MaybeUninit;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SlotMapKey(usize);

pub type SlotMap<T> = details::MetaSlotMap<
    T,
    OwningPointer<MaybeUninit<Option<T>>>,
    OwningPointer<MaybeUninit<usize>>,
>;
pub type RelocatableSlotMap<T> = details::MetaSlotMap<
    T,
    RelocatablePointer<MaybeUninit<Option<T>>>,
    RelocatablePointer<MaybeUninit<usize>>,
>;

const INVALID_KEY: usize = usize::MAX;

#[doc(hidden)]
pub mod details {
    use super::*;

    pub struct Iter<
        'slotmap,
        T,
        DataPtrType: PointerTrait<MaybeUninit<Option<T>>>,
        IdxPtrType: PointerTrait<MaybeUninit<usize>>,
    > {
        slotmap: &'slotmap MetaSlotMap<T, DataPtrType, IdxPtrType>,
        key: SlotMapKey,
    }

    impl<
            'slotmap,
            T,
            DataPtrType: PointerTrait<MaybeUninit<Option<T>>>,
            IdxPtrType: PointerTrait<MaybeUninit<usize>>,
        > Iterator for Iter<'slotmap, T, DataPtrType, IdxPtrType>
    {
        type Item = (SlotMapKey, &'slotmap T);

        fn next<'this>(&'this mut self) -> Option<Self::Item> {
            if let Some((key, value)) = self.slotmap.next(self.key) {
                self.key.0 = key.0 + 1;
                Some((key, value))
            } else {
                None
            }
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct MetaSlotMap<
        T,
        DataPtrType: PointerTrait<MaybeUninit<Option<T>>>,
        IdxPtrType: PointerTrait<MaybeUninit<usize>>,
    > {
        idx_to_data: MetaVec<usize, IdxPtrType>,
        idx_to_data_next_free_index: MetaQueue<usize, IdxPtrType>,
        data: MetaVec<Option<T>, DataPtrType>,
        data_next_free_index: MetaQueue<usize, IdxPtrType>,
    }

    impl<
            T,
            DataPtrType: PointerTrait<MaybeUninit<Option<T>>>,
            IdxPtrType: PointerTrait<MaybeUninit<usize>>,
        > MetaSlotMap<T, DataPtrType, IdxPtrType>
    {
        fn next(&self, start: SlotMapKey) -> Option<(SlotMapKey, &T)> {
            let idx_to_data = &self.idx_to_data;

            for n in start.0..idx_to_data.len() {
                let data_idx = self.idx_to_data[n];
                if data_idx != INVALID_KEY {
                    return Some((
                        SlotMapKey(n),
                        self.data[data_idx].as_ref().expect(
                            "By contract, data contains a value when idx_to_data contains a value",
                        ),
                    ));
                }
            }

            None
        }

        pub(crate) unsafe fn initialize_data_structures(&mut self) {
            self.idx_to_data.fill(INVALID_KEY);
            self.data.fill_with(|| None);
            for n in 0..self.capacity_impl() {
                self.idx_to_data_next_free_index.push_impl(n);
                self.data_next_free_index.push_impl(n);
            }
        }

        pub(crate) unsafe fn iter_impl(&self) -> Iter<T, DataPtrType, IdxPtrType> {
            Iter {
                slotmap: self,
                key: SlotMapKey(0),
            }
        }

        pub(crate) unsafe fn get_impl(&self, key: SlotMapKey) -> Option<&T> {
            match self.idx_to_data[key.0] {
                INVALID_KEY => None,
                n => Some(self.data[n].as_ref().expect(
                    "data and idx_to_data correspond and this value must be always available.",
                )),
            }
        }

        pub(crate) unsafe fn get_mut_impl(&mut self, key: SlotMapKey) -> Option<&mut T> {
            match self.idx_to_data[key.0] {
                INVALID_KEY => None,
                n => Some(self.data[n].as_mut().expect(
                    "data and idx_to_data correspond and this value must be always available.",
                )),
            }
        }

        pub(crate) unsafe fn insert_impl(&mut self, value: T) -> Option<SlotMapKey> {
            match self.idx_to_data_next_free_index.pop_impl() {
                None => None,
                Some(key) => {
                    let key = SlotMapKey(key);
                    self.insert_at_impl(key, value);
                    Some(key)
                }
            }
        }

        pub(crate) unsafe fn insert_at_impl(&mut self, key: SlotMapKey, value: T) -> bool {
            if key.0 > self.capacity_impl() {
                return false;
            }

            let data_idx = self.idx_to_data[key.0];
            if data_idx != INVALID_KEY {
                self.data[data_idx] = Some(value);
                true
            } else {
                let n = self.data_next_free_index.pop_impl().expect("data and idx_to_data correspond and there must be always a free index available.");
                self.idx_to_data[key.0] = n;
                self.data[n] = Some(value);
                false
            }
        }

        pub(crate) unsafe fn remove_impl(&mut self, key: SlotMapKey) -> bool {
            if key.0 > self.idx_to_data.len() {
                return false;
            }

            let data_idx = self.idx_to_data[key.0];
            if data_idx != INVALID_KEY {
                self.data[data_idx].take();
                self.data_next_free_index.push_impl(data_idx);
                self.idx_to_data_next_free_index.push_impl(key.0);
                self.idx_to_data[key.0] = INVALID_KEY;
                true
            } else {
                false
            }
        }

        pub(crate) unsafe fn len_impl(&self) -> usize {
            self.capacity_impl() - self.idx_to_data_next_free_index.len()
        }

        pub(crate) unsafe fn capacity_impl(&self) -> usize {
            self.idx_to_data.capacity()
        }

        pub(crate) unsafe fn is_empty_impl(&self) -> bool {
            self.len_impl() == 0
        }

        pub(crate) unsafe fn is_full_impl(&self) -> bool {
            self.len_impl() == self.capacity_impl()
        }
    }

    impl<T> RelocatableContainer
        for MetaSlotMap<
            T,
            RelocatablePointer<MaybeUninit<Option<T>>>,
            RelocatablePointer<MaybeUninit<usize>>,
        >
    {
        unsafe fn new(capacity: usize, distance_to_data: isize) -> Self {
            let mut new_self = Self {
                idx_to_data: RelocatableVec::new(capacity, distance_to_data),
                idx_to_data_next_free_index: RelocatableQueue::new(
                    capacity,
                    distance_to_data
                        + RelocatableVec::<usize>::const_memory_size(capacity) as isize,
                ),
                data: RelocatableVec::new(
                    capacity,
                    distance_to_data
                        + RelocatableVec::<usize>::const_memory_size(capacity) as isize
                        + RelocatableQueue::<usize>::const_memory_size(capacity) as isize,
                ),
                data_next_free_index: RelocatableQueue::new(
                    capacity,
                    distance_to_data
                        + RelocatableVec::<usize>::const_memory_size(capacity) as isize
                        + RelocatableQueue::<usize>::const_memory_size(capacity) as isize
                        + RelocatableVec::<Option<T>>::const_memory_size(capacity) as isize,
                ),
            };
            new_self.initialize_data_structures();
            new_self
        }

        unsafe fn new_uninit(capacity: usize) -> Self {
            Self {
                idx_to_data: RelocatableVec::new_uninit(capacity),
                idx_to_data_next_free_index: RelocatableQueue::new_uninit(capacity),
                data: RelocatableVec::new_uninit(capacity),
                data_next_free_index: RelocatableQueue::new_uninit(capacity),
            }
        }

        unsafe fn init<Allocator: iceoryx2_bb_elementary::allocator::BaseAllocator>(
            &mut self,
            allocator: &Allocator,
        ) -> Result<(), iceoryx2_bb_elementary::allocator::AllocationError> {
            let msg = "Unable to initialize RelocatableSlotMap";
            fail!(from "RelocatableSlotMap::init()",
                  when self.idx_to_data.init(allocator),
                  "{msg} since the underlying idx_to_data vector could not be initialized.");
            fail!(from "RelocatableSlotMap::init()",
                  when self.idx_to_data_next_free_index.init(allocator),
                  "{msg} since the underlying idx_to_data_next_free_index queue could not be initialized.");
            fail!(from "RelocatableSlotMap::init()",
                  when self.data.init(allocator),
                  "{msg} since the underlying data vector could not be initialized.");
            fail!(from "RelocatableSlotMap::init()",
                  when self.data_next_free_index.init(allocator),
                  "{msg} since the underlying data_next_free_index queue could not be initialized.");

            self.initialize_data_structures();
            Ok(())
        }

        fn memory_size(capacity: usize) -> usize {
            Self::const_memory_size(capacity)
        }
    }

    impl<T>
        MetaSlotMap<
            T,
            RelocatablePointer<MaybeUninit<Option<T>>>,
            RelocatablePointer<MaybeUninit<usize>>,
        >
    {
        pub const fn const_memory_size(capacity: usize) -> usize {
            RelocatableVec::<usize>::const_memory_size(capacity)
                + RelocatableQueue::<usize>::const_memory_size(capacity)
                + RelocatableVec::<Option<T>>::const_memory_size(capacity)
                + RelocatableQueue::<usize>::const_memory_size(capacity)
        }
    }

    impl<T> MetaSlotMap<T, OwningPointer<MaybeUninit<Option<T>>>, OwningPointer<MaybeUninit<usize>>> {
        pub fn new(capacity: usize) -> Self {
            let mut new_self = Self {
                idx_to_data: MetaVec::new(capacity),
                idx_to_data_next_free_index: MetaQueue::new(capacity),
                data: MetaVec::new(capacity),
                data_next_free_index: MetaQueue::new(capacity),
            };
            unsafe { new_self.initialize_data_structures() };
            new_self
        }

        pub fn iter(
            &self,
        ) -> Iter<T, OwningPointer<MaybeUninit<Option<T>>>, OwningPointer<MaybeUninit<usize>>>
        {
            unsafe { self.iter_impl() }
        }

        pub fn get(&self, key: SlotMapKey) -> Option<&T> {
            unsafe { self.get_impl(key) }
        }

        pub fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
            unsafe { self.get_mut_impl(key) }
        }

        pub fn insert(&mut self, value: T) -> Option<SlotMapKey> {
            unsafe { self.insert_impl(value) }
        }

        pub fn insert_at(&mut self, key: SlotMapKey, value: T) -> bool {
            unsafe { self.insert_at_impl(key, value) }
        }

        pub fn remove(&mut self, key: SlotMapKey) -> bool {
            unsafe { self.remove_impl(key) }
        }

        pub fn len(&self) -> usize {
            unsafe { self.len_impl() }
        }

        pub fn capacity(&self) -> usize {
            unsafe { self.capacity_impl() }
        }

        pub fn is_empty(&self) -> bool {
            unsafe { self.is_empty_impl() }
        }

        pub fn is_full(&self) -> bool {
            unsafe { self.is_full_impl() }
        }
    }

    impl<T>
        MetaSlotMap<
            T,
            RelocatablePointer<MaybeUninit<Option<T>>>,
            RelocatablePointer<MaybeUninit<usize>>,
        >
    {
        pub unsafe fn iter(
            &self,
        ) -> Iter<
            T,
            RelocatablePointer<MaybeUninit<Option<T>>>,
            RelocatablePointer<MaybeUninit<usize>>,
        > {
            self.iter_impl()
        }

        pub unsafe fn get(&self, key: SlotMapKey) -> Option<&T> {
            self.get_impl(key)
        }

        pub unsafe fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
            self.get_mut_impl(key)
        }

        pub unsafe fn insert(&mut self, value: T) -> Option<SlotMapKey> {
            self.insert_impl(value)
        }

        pub unsafe fn insert_at(&mut self, key: SlotMapKey, value: T) -> bool {
            self.insert_at_impl(key, value)
        }

        pub unsafe fn remove(&mut self, key: SlotMapKey) -> bool {
            self.remove_impl(key)
        }

        pub unsafe fn len(&self) -> usize {
            self.len_impl()
        }

        pub unsafe fn capacity(&self) -> usize {
            self.capacity_impl()
        }

        pub unsafe fn is_empty(&self) -> bool {
            self.is_empty_impl()
        }

        pub unsafe fn is_full(&self) -> bool {
            self.is_full_impl()
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FixedSizeSlotMap<T, const CAPACITY: usize> {
    state: RelocatableSlotMap<T>,
    _idx_to_data: [usize; CAPACITY],
    _idx_to_data_next_free_index: [usize; CAPACITY],
    _data: [Option<T>; CAPACITY],
    _data_next_free_index: [usize; CAPACITY],
}

impl<T, const CAPACITY: usize> Default for FixedSizeSlotMap<T, CAPACITY> {
    fn default() -> Self {
        let mut new_self = Self {
            _idx_to_data: core::array::from_fn(|_| INVALID_KEY),
            _idx_to_data_next_free_index: core::array::from_fn(|_| 0),
            _data: core::array::from_fn(|_| None),
            _data_next_free_index: core::array::from_fn(|_| 0),
            state: Self::initialize_state(),
        };

        let allocator = BumpAllocator::new(core::ptr::addr_of!(new_self._idx_to_data) as usize);
        unsafe {
            new_self
                .state
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        new_self
    }
}

impl<T, const CAPACITY: usize> FixedSizeSlotMap<T, CAPACITY> {
    fn initialize_state() -> RelocatableSlotMap<T> {
        unsafe { RelocatableSlotMap::new_uninit(CAPACITY) }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(
        &self,
    ) -> details::Iter<
        T,
        RelocatablePointer<MaybeUninit<Option<T>>>,
        RelocatablePointer<MaybeUninit<usize>>,
    > {
        unsafe { self.state.iter_impl() }
    }

    pub fn get(&self, key: SlotMapKey) -> Option<&T> {
        unsafe { self.state.get_impl(key) }
    }

    pub fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
        unsafe { self.state.get_mut_impl(key) }
    }

    pub fn insert(&mut self, value: T) -> Option<SlotMapKey> {
        unsafe { self.state.insert_impl(value) }
    }

    pub fn insert_at(&mut self, key: SlotMapKey, value: T) -> bool {
        unsafe { self.state.insert_at_impl(key, value) }
    }

    pub fn remove(&mut self, key: SlotMapKey) -> bool {
        unsafe { self.state.remove_impl(key) }
    }

    pub fn len(&self) -> usize {
        unsafe { self.state.len_impl() }
    }

    pub fn capacity(&self) -> usize {
        unsafe { self.state.capacity_impl() }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { self.state.is_empty_impl() }
    }

    pub fn is_full(&self) -> bool {
        unsafe { self.state.is_full_impl() }
    }
}
