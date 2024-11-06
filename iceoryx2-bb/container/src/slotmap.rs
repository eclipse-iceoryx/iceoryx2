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

use crate::queue::details::Queue;
use crate::vec::details::RelocatableVec;
use iceoryx2_bb_elementary::owning_pointer::OwningPointer;
use iceoryx2_bb_elementary::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use std::mem::MaybeUninit;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SlotMapKey(usize);

pub type SlotMap<T> = details::RelocatableSlotMap<
    T,
    OwningPointer<MaybeUninit<Option<T>>>,
    OwningPointer<MaybeUninit<usize>>,
>;
pub type RelocatableSlotMap<T> = details::RelocatableSlotMap<
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
        slotmap: &'slotmap RelocatableSlotMap<T, DataPtrType, IdxPtrType>,
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
    pub struct RelocatableSlotMap<
        T,
        DataPtrType: PointerTrait<MaybeUninit<Option<T>>>,
        IdxPtrType: PointerTrait<MaybeUninit<usize>>,
    > {
        idx_to_data: RelocatableVec<usize, IdxPtrType>,
        idx_to_data_next_free_index: Queue<usize, IdxPtrType>,
        data: RelocatableVec<Option<T>, DataPtrType>,
        data_next_free_index: Queue<usize, IdxPtrType>,
    }

    impl<
            T,
            DataPtrType: PointerTrait<MaybeUninit<Option<T>>>,
            IdxPtrType: PointerTrait<MaybeUninit<usize>>,
        > RelocatableSlotMap<T, DataPtrType, IdxPtrType>
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
    }

    impl<T>
        RelocatableSlotMap<
            T,
            OwningPointer<MaybeUninit<Option<T>>>,
            OwningPointer<MaybeUninit<usize>>,
        >
    {
        pub fn new(capacity: usize) -> Self {
            let mut new_self = Self {
                idx_to_data: RelocatableVec::new(capacity),
                idx_to_data_next_free_index: Queue::new(capacity),
                data: RelocatableVec::new(capacity),
                data_next_free_index: Queue::new(capacity),
            };

            new_self.idx_to_data.fill(INVALID_KEY);
            new_self.data.fill_with(|| None);
            for n in 0..capacity {
                new_self.idx_to_data_next_free_index.push(n);
                new_self.data_next_free_index.push(n);
            }
            new_self
        }

        pub fn iter(
            &self,
        ) -> Iter<T, OwningPointer<MaybeUninit<Option<T>>>, OwningPointer<MaybeUninit<usize>>>
        {
            Iter {
                slotmap: self,
                key: SlotMapKey(0),
            }
        }

        pub fn get(&self, key: SlotMapKey) -> Option<&T> {
            match self.idx_to_data[key.0] {
                INVALID_KEY => None,
                n => Some(self.data[n].as_ref().expect(
                    "data and idx_to_data correspond and this value must be always available.",
                )),
            }
        }

        pub fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
            match self.idx_to_data[key.0] {
                INVALID_KEY => None,
                n => Some(self.data[n].as_mut().expect(
                    "data and idx_to_data correspond and this value must be always available.",
                )),
            }
        }

        pub fn insert(&mut self, value: T) -> Option<SlotMapKey> {
            match self.idx_to_data_next_free_index.pop() {
                None => None,
                Some(key) => {
                    let key = SlotMapKey(key);
                    self.insert_at(key, value);
                    Some(key)
                }
            }
        }

        pub fn insert_at(&mut self, key: SlotMapKey, value: T) -> bool {
            if key.0 > self.capacity() {
                return false;
            }

            let data_idx = self.idx_to_data[key.0];
            if data_idx != INVALID_KEY {
                self.data[data_idx] = Some(value);
                true
            } else {
                let n = self.data_next_free_index.pop().expect("data and idx_to_data correspond and there must be always a free index available.");
                self.idx_to_data[key.0] = n;
                self.data[n] = Some(value);
                false
            }
        }

        pub fn remove(&mut self, key: SlotMapKey) -> bool {
            if key.0 > self.idx_to_data.len() {
                return false;
            }

            let data_idx = self.idx_to_data[key.0];
            if data_idx != INVALID_KEY {
                self.data[data_idx].take();
                self.data_next_free_index.push(data_idx);
                self.idx_to_data_next_free_index.push(key.0);
                self.idx_to_data[key.0] = INVALID_KEY;
                true
            } else {
                false
            }
        }

        pub fn len(&self) -> usize {
            self.capacity() - self.data_next_free_index.len()
        }

        pub fn capacity(&self) -> usize {
            self.idx_to_data.capacity()
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        pub fn is_full(&self) -> bool {
            self.len() == self.capacity()
        }
    }
}
