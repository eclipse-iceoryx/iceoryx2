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

//! A [`ResizableSharedMemory`] is identified by a name and allows multiple processes to share
//! memory between them (inter-process memory). One process owns the [`ResizableSharedMemory`]
//! which can be created via the [`ResizableSharedMemoryBuilder`] and many processes can have a
//! [`ResizableSharedMemoryView`] that can be constructed via [`ResizableSharedMemoryViewBuilder`].
//!
//! The [`ResizableSharedMemoryView`] never owns the [`ResizableSharedMemory`] and has only
//! read-only access to it while the [`ResizableSharedMemory`] can use
//! [`ResizableSharedMemory::allocate()`] to acquire memory and distribute it between the
//! [`ResizableSharedMemoryView`]s.
//!
//! Whenever the [`ResizableSharedMemoryView`] receives an offset it must be registered via
//! [`ResizableSharedMemoryView::register_and_translate_offset()`] and unregistered via
//! [`ResizableSharedMemoryView::unregister_offset()`]. As soon as the [`ResizableSharedMemory`]
//! calls [`ResizableSharedMemory::deallocate()`] unused [`SharedMemory`] segments may be recycled.
//!
//! # Example
//!
//! ```
//! // owner of the ResizableSharedMemory
//! use iceoryx2_cal::resizable_shared_memory::*;
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_cal::shm_allocator::ShmAllocator;
//! use iceoryx2_cal::shared_memory::SharedMemory;
//! use iceoryx2_cal::named_concept::*;
//! use core::alloc::Layout;
//! use core::time::Duration;
//!
//! fn example<Alloc: ShmAllocator, Shm: SharedMemory<Alloc>, Memory: ResizableSharedMemory<Alloc, Shm>>(
//!     name: &FileName
//! ) {
//!     // owner process creates a new memory
//!     let memory = Memory::MemoryBuilder::new(name)
//!         // hint to the underlying allocator that we need up to 128 chunks of memory
//!         //
//!         // note: as soon as there are more than 128 chunks requested a new resized segment is
//!         //       created
//!         .max_number_of_chunks_hint(128)
//!         // hint to the underlying allocator that the chunks are not exceeding the Layout of
//!         // [`u16`]
//!         //
//!         // note: as soon as there is a chunk requested that exceeds the provided layout hint a
//!         //       new resized segment is created
//!         .max_chunk_layout_hint(Layout::new::<u32>())
//!         // defines the strategy how segments are resized
//!         .allocation_strategy(AllocationStrategy::PowerOfTwo)
//!         .create().unwrap();
//!
//!     let chunk = memory.allocate(Layout::new::<u16>()).unwrap();
//!     // store the value 123 in the newly allocated chunk
//!     unsafe { (chunk.data_ptr as *mut u16).write(123) };
//!
//!     // since this exceeds the the chunk layout hint, the underlying segment is resized
//!     // following the provided allocation strategy
//!     let another_chunk = memory.allocate(Layout::new::<u64>()).unwrap();
//!     // release allocated chunk
//!     unsafe { memory.deallocate(another_chunk.offset, Layout::new::<u64>()) };
//!
//!
//!     let view = Memory::ViewBuilder::new(name)
//!         // defines how long the builder shall wait to open the corresponding segments when
//!         // the creator is concurrently creating them.
//!         .timeout(Duration::from_secs(1))
//!         .open().unwrap();
//!
//!     // before we can consume the received offset we need to translate it into our local
//!     // process space
//!     // this operation also maps unmapped segments into the process space if required.
//!     let ptr = unsafe { view.register_and_translate_offset(chunk.offset).unwrap() };
//!     println!("received {}", unsafe {*(ptr as *const u16)});
//!
//!     // when we are finished consuming the memory we need to unregister the offset. this
//!     // unmaps segments that are no longer used.
//!     unsafe { view.unregister_offset(chunk.offset) };
//! }
//! ```

pub mod dynamic;
pub mod recommended;

pub use crate::shm_allocator::{pool_allocator::PoolAllocator, AllocationStrategy};

use core::alloc::Layout;
use core::fmt::Debug;
use core::time::Duration;

use iceoryx2_bb_elementary::enum_gen;

use crate::named_concept::*;
use crate::shared_memory::{
    SegmentId, SharedMemory, SharedMemoryCreateError, SharedMemoryOpenError, ShmPointer,
};
use crate::shm_allocator::{PointerOffset, ShmAllocationError, ShmAllocator};

enum_gen! {
/// Defines all erros that can occur when calling [`ResizableSharedMemory::allocate()`]
///
/// The [`ResizableSharedMemory`] cannot be resized indefinitely. If the resize limit is hit
/// this error will be returned. It can be mitigated by providing a better
/// [`ResizableSharedMemoryBuilder::max_number_of_chunks_hint()`] or
/// [`ResizableSharedMemoryBuilder::max_chunk_layout_hint()`].
    ResizableShmAllocationError
  entry:
    MaxReallocationsReached
  mapping:
    ShmAllocationError,
    SharedMemoryCreateError
}

/// Creates a [`ResizableSharedMemoryView`] to an existing [`ResizableSharedMemory`] and maps the
/// [`ResizableSharedMemory`] read-only into the process space.
pub trait ResizableSharedMemoryViewBuilder<
    Allocator: ShmAllocator,
    Shm: SharedMemory<Allocator>,
    ResizableShm: ResizableSharedMemory<Allocator, Shm>,
    ResizableShmView: ResizableSharedMemoryView<Allocator, Shm>,
>: NamedConceptBuilder<ResizableShm> + Debug
{
    /// The timeout defines how long the
    /// [`SharedMemoryBuilder`](crate::shared_memory::SharedMemoryBuilder) should wait for
    /// [`SharedMemoryBuilder::create()`](crate::shared_memory::SharedMemoryBuilder::create()) to finialize
    /// the initialization. This is required when the [`SharedMemory`] is created and initialized
    /// concurrently from another process. By default it is set to [`Duration::ZERO`] for no
    /// timeout.
    fn timeout(self, value: Duration) -> Self;

    /// Opens already existing [`SharedMemory`]. If it does not exist or the initialization is not
    /// yet finished the method will fail.
    fn open(self) -> Result<ResizableShmView, SharedMemoryOpenError>;
}

/// Creates a new [`ResizableSharedMemory`] which the process will own. As soon as the
/// corresponding object goes out-of-scope the underlying [`SharedMemory`] resources will be
/// removed.
pub trait ResizableSharedMemoryBuilder<
    Allocator: ShmAllocator,
    Shm: SharedMemory<Allocator>,
    ResizableShm: ResizableSharedMemory<Allocator, Shm>,
>: NamedConceptBuilder<ResizableShm> + Debug
{
    /// Provides an initial hint to the underlying [`ShmAllocator`] on how large the largest chunk
    /// will be. If the chunk exceeds the hinted [`Layout`] a new [`SharedMemory`] segment is
    /// acquired to satisfy the memory needs.
    fn max_chunk_layout_hint(self, value: Layout) -> Self;

    /// Provides an initial hint to the underlying [`ShmAllocator`] on how many chunks at most will
    /// be used in parallel. If the number of chunk exceeds the hint a new [`SharedMemory`] segment
    /// is acquired to satisfy the memory needs.
    fn max_number_of_chunks_hint(self, value: usize) -> Self;

    /// Defines the [`AllocationStrategy`] that is pursued when a new [`SharedMemory`] segment is
    /// acquired.
    fn allocation_strategy(self, value: AllocationStrategy) -> Self;

    /// Creates new [`SharedMemory`]. If it already exists the method will fail.
    fn create(self) -> Result<ResizableShm, SharedMemoryCreateError>;
}

/// A read-only view to a [`ResizableSharedMemory`]. Can be created by arbitrary many processes.
pub trait ResizableSharedMemoryView<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>:
    Debug + Send
{
    /// Registers a received [`PointerOffset`] at the [`ResizableSharedMemoryView`] and returns the
    /// absolut pointer to the data. If the segment of the received [`PointerOffset`] was not yet
    /// mapped into the processes space, it will be opened and mapped. If this fails a
    /// [`SharedMemoryOpenError`] is returned.
    ///
    /// # Safety
    ///
    ///   * This function shall be called exactly once for a received [`PointerOffset`]
    unsafe fn register_and_translate_offset(
        &self,
        offset: PointerOffset,
    ) -> Result<*const u8, SharedMemoryOpenError>;

    /// Unregisters a received [`PointerOffset`] that was previously registered.
    ///
    /// # Safety
    ///
    ///  * [`ResizableSharedMemoryView::register_and_translate_offset()`] must have been called
    ///    with the same [`PointerOffset`] before calling this function.
    ///  * This function must be called before a registered [`PointerOffset`] goes out-of-scope.
    ///  * This function must be called at most once for any received [`PointerOffset`]
    unsafe fn unregister_offset(&self, offset: PointerOffset);

    /// Returns the number of active [`SharedMemory`] segments.
    fn number_of_active_segments(&self) -> usize;
}

/// The [`ResizableSharedMemory`] can be only owned by exactly one process that is allowed to
/// [`ResizableSharedMemory::allocate()`] memory and distribute the memory to all
/// [`ResizableSharedMemoryView`]s.
pub trait ResizableSharedMemory<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>:
    Sized + NamedConcept + NamedConceptMgmt + Debug + Send
{
    /// Type alias to the [`ResizableSharedMemoryViewBuilder`] to open a
    /// [`ResizableSharedMemoryView`] to an existing [`ResizableSharedMemory`].
    type ViewBuilder: ResizableSharedMemoryViewBuilder<Allocator, Shm, Self, Self::View>;

    /// Type alias to the [`ResizableSharedMemoryBuilder`] to create a new [`ResizableSharedMemory`].
    type MemoryBuilder: ResizableSharedMemoryBuilder<Allocator, Shm, Self>;

    /// Type alias to the [`ResizableSharedMemoryView`] to open an existing
    /// [`ResizableSharedMemory`] as read-only.
    type View: ResizableSharedMemoryView<Allocator, Shm>;

    /// Returns how many reallocations the [`ResizableSharedMemory`] supports. If the number is
    /// exceeded any call to [`ResizableSharedMemory::allocate()`] that requires a resize of the
    /// underlying [`SharedMemory`] segments will fail.
    fn max_number_of_reallocations() -> usize;

    /// Returns the number of active [`SharedMemory`] segments.
    fn number_of_active_segments(&self) -> usize;

    /// Allocates a new piece of [`SharedMemory`] if the provided [`Layout`] exceeds the current
    /// supported [`Layout`], the memory would be out-of-memory or the number of chunks exceeds the
    /// current supported amount of chunks, a new [`SharedMemory`] segment will be created. If this
    /// fails an [`SharedMemoryCreateError`] will be returned.
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<ShmPointer, ResizableShmAllocationError>;

    /// Release previously allocated memory
    ///
    /// # Safety
    ///
    ///  * the offset must be acquired with [`SharedMemory::allocate()`] - extracted from the
    ///    [`ShmPointer`]
    ///  * the layout must be identical to the one used in [`SharedMemory::allocate()`]
    unsafe fn deallocate(&self, offset: PointerOffset, layout: core::alloc::Layout);
}

pub trait ResizableSharedMemoryForPoolAllocator<Shm: SharedMemory<PoolAllocator>>:
    ResizableSharedMemory<PoolAllocator, Shm>
{
    /// Release previously allocated memory
    ///
    /// # Safety
    ///
    ///  * the offset must be acquired with [`SharedMemory::allocate()`] - extracted from the
    ///    [`ShmPointer`]
    unsafe fn deallocate_bucket(&self, offset: PointerOffset);

    /// Returns the bucket size of the corresponding [`PoolAllocator`]
    fn bucket_size(&self, segment_id: SegmentId) -> usize;
}
