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
use iceoryx2_bb_flatbuffers::AllocationStrategy;
use iceoryx2_bb_memory::pool_allocator::ContentPlacement;
use iceoryx2_cal::{
    arc_sync_policy::{ArcSyncPolicy, ArcSyncPolicyCreationError},
    shared_memory::ShmPointer,
    shm_allocator::PointerOffset,
};
use iceoryx2_log::fail;

use crate::port::details::chunk::ChunkMut;
use crate::port::details::port_shared_state::PortSharedState;

#[derive(Debug)]
pub struct ChunkMutInnerSharedState<Service: crate::service::Service, T: PortSharedState> {
    port_shared_state: Service::ArcThreadSafetyPolicy<T>,
    offset_to_chunk: AtomicU64,
    shm_raw_ptr: AtomicUsize,
    slice_len: AtomicUsize,
}

impl<Service: crate::service::Service, T: PortSharedState> ChunkMutInnerSharedState<Service, T> {
    pub fn offset_to_chunk(&self) -> PointerOffset {
        PointerOffset::from_value(self.offset_to_chunk.load(Ordering::Relaxed))
    }
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

#[derive(Debug)]
pub struct ChunkMutSharedState<Service: crate::service::Service, T: PortSharedState> {
    pub(crate) state: Service::ArcThreadSafetyPolicy<ChunkMutInnerSharedState<Service, T>>,
}

impl<Service: crate::service::Service, T: PortSharedState> Clone
    for ChunkMutSharedState<Service, T>
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<Service: crate::service::Service, T: PortSharedState> ChunkMutSharedState<Service, T> {
    pub fn new(
        port_shared_state: &Service::ArcThreadSafetyPolicy<T>,
        chunk: &ChunkMut,
    ) -> Result<Self, ArcSyncPolicyCreationError> {
        let state = match Service::ArcThreadSafetyPolicy::new(ChunkMutInnerSharedState {
            port_shared_state: port_shared_state.clone(),
            offset_to_chunk: AtomicU64::new(chunk.offset().as_value()),
            shm_raw_ptr: AtomicUsize::new(chunk.header_ptr() as usize),
            slice_len: AtomicUsize::new(chunk.size()),
        }) {
            Ok(v) => v,
            Err(e) => {
                fail!(from "ChunkMutSharedState::new()", with e,
                      "Unable to create new shared state since the thread safety policy creation failed. [{e:?}]");
            }
        };

        Ok(Self { state })
    }

    pub fn clone_shared_port_state(&self) -> Service::ArcThreadSafetyPolicy<T> {
        self.state.lock().port_shared_state.clone()
    }

    pub fn call<SuccVal, ErrVal, F: FnOnce(&T) -> Result<SuccVal, ErrVal>>(
        &self,
        callback: F,
    ) -> Result<SuccVal, ErrVal> {
        callback(&self.state.lock().port_shared_state.lock())
    }

    pub fn slice_len(&self) -> usize {
        self.state.lock().slice_len.load(Ordering::Relaxed)
    }

    pub fn allocation_strategy(&self) -> AllocationStrategy {
        self.state
            .lock()
            .port_shared_state
            .lock()
            .allocation_strategy()
    }

    pub fn header_len(&self) -> usize {
        self.state.lock().port_shared_state.lock().header_len()
    }

    pub fn payload_size(&self) -> usize {
        self.state.lock().port_shared_state.lock().payload_size()
    }

    pub fn update_chunk_pointers_to_reallocated_layout(&self) -> ChunkMut {
        let state = self.state.lock();
        let port_state = state.port_shared_state.lock();

        let message_type_details = port_state.message_type_details();
        let header = state.shm_raw_ptr.load(Ordering::Relaxed) as *mut u8;
        let user_header = message_type_details
            .user_header_ptr_from_header(header)
            .cast_mut();
        let payload = message_type_details
            .payload_ptr_from_header(header)
            .cast_mut();
        let header_len = message_type_details.all_headers_len();

        ChunkMut {
            offset: PointerOffset::from_value(state.offset_to_chunk.load(Ordering::Relaxed)),
            size: state.slice_len.load(Ordering::Relaxed) + header_len,
            header,
            user_header,
            payload,
        }
    }

    #[doc(hidden)]
    /// Required for language bindings
    pub fn __internal_override_slice_len(&self, value: usize) {
        self.state.lock().slice_len.store(value, Ordering::Relaxed)
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
