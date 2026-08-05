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

use core::alloc::Layout;

use iceoryx2_bb_concurrency::atomic::{AtomicU64, AtomicUsize, Ordering};
use iceoryx2_bb_elementary_traits::allocator::{AllocationGrowError, Grow};
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_memory::pool_allocator::ContentPlacement;
use iceoryx2_cal::{
    arc_sync_policy::{ArcSyncPolicy, ArcSyncPolicyCreationError},
    shared_memory::ShmPointer,
    shm_allocator::PointerOffset,
};
use iceoryx2_log::fail;

use crate::port::details::port_shared_state::PortSharedState;

#[derive(Debug)]
pub(crate) struct ChunkMutInnerSharedState<Service: crate::service::Service, T: PortSharedState> {
    pub(crate) port_shared_state: Service::ArcThreadSafetyPolicy<T>,
    pub(crate) offset_to_chunk: AtomicU64,
    pub(crate) shm_raw_ptr: AtomicUsize,
    pub(crate) slice_len: AtomicUsize,
}

unsafe impl<Service: crate::service::Service, T: PortSharedState> Send
    for ChunkMutInnerSharedState<Service, T>
{
}

impl<Service: crate::service::Service, T: PortSharedState> Abandonable
    for ChunkMutInnerSharedState<Service, T>
{
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            Service::ArcThreadSafetyPolicy::<T>::abandon_in_place(core::ptr::NonNull::from_mut(
                &mut this.port_shared_state,
            ));
        }
    }
}

impl<Service: crate::service::Service, T: PortSharedState> Drop
    for ChunkMutInnerSharedState<Service, T>
{
    fn drop(&mut self) {
        self.port_shared_state
            .lock()
            .return_loan(PointerOffset::from_value(
                self.offset_to_chunk.load(Ordering::Relaxed),
            ));
    }
}

#[derive(Debug, Clone)]
pub struct ChunkMutSharedState<Service: crate::service::Service, T: PortSharedState> {
    pub(crate) state: Service::ArcThreadSafetyPolicy<ChunkMutInnerSharedState<Service, T>>,
}

impl<Service: crate::service::Service, T: PortSharedState> ChunkMutSharedState<Service, T> {
    pub(crate) fn new(
        port_shared_state: &Service::ArcThreadSafetyPolicy<T>,
        pointer_to_chunk: ShmPointer,
        underlying_slice_len: usize,
    ) -> Result<Self, ArcSyncPolicyCreationError> {
        let state = match Service::ArcThreadSafetyPolicy::new(ChunkMutInnerSharedState {
            port_shared_state: port_shared_state.clone(),
            offset_to_chunk: AtomicU64::new(pointer_to_chunk.offset.as_value()),
            shm_raw_ptr: AtomicUsize::new(pointer_to_chunk.data_ptr as usize),
            slice_len: AtomicUsize::new(underlying_slice_len),
        }) {
            Ok(v) => v,
            Err(e) => {
                fail!(from "ChunkMutSharedState::new()", with e,
                      "Unable to create new shared state since the thread safety policy creation failed. [{e:?}]");
            }
        };

        Ok(Self { state })
    }
}

impl<Service: crate::service::Service, T: PortSharedState> Grow<ShmPointer>
    for ChunkMutSharedState<Service, T>
{
    unsafe fn grow(
        &self,
        ptr: ShmPointer,
        old_layout: Layout,
        new_layout: Layout,
        content_placement: ContentPlacement,
    ) -> Result<ShmPointer, AllocationGrowError> {
        let state = self.state.lock();
        let ptr = unsafe {
            state
                .port_shared_state
                .lock()
                .grow(ptr, old_layout, new_layout, content_placement)
        }?;

        state
            .offset_to_chunk
            .store(ptr.offset.as_value(), Ordering::Relaxed);
        state
            .shm_raw_ptr
            .store(ptr.data_ptr as usize, Ordering::Relaxed);
        state.slice_len.store(new_layout.size(), Ordering::Relaxed);

        Ok(ptr)
    }
}

impl<Service: crate::service::Service, T: PortSharedState> ChunkMutSharedState<Service, T> {
    pub(crate) fn slice_len(&self) -> usize {
        self.state.lock().slice_len.load(Ordering::Relaxed)
    }
}
