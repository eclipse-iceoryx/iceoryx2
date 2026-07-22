// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! ## Typed API
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//! #     .publish_subscribe::<u64>()
//! #     .open_or_create()?;
//! #
//! # let publisher = service.publisher_builder().create()?;
//!
//! // initializes the payload with `Default::default()`
//! let mut sample = publisher.loan()?;
//! // override contents with 456 because its fun
//! *sample.payload_mut() = 456;
//!
//! println!("publisher port id: {:?}", sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Slice API
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//! #     .publish_subscribe::<[usize]>()
//! #     .create()?;
//! #
//! # let publisher = service.publisher_builder().initial_max_slice_len(16).create()?;
//!
//! let slice_length = 12;
//! // initializes every element of the slice with `Default::default()`
//! let mut sample = publisher.loan_slice(slice_length)?;
//! // override the content of the first element with 42
//! sample.payload_mut()[0] = 42;
//!
//! println!("publisher port id: {:?}", sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::port::details::chunk::ChunkMut;
use crate::service::marker::Flatbuffer;
use crate::{
    port::SendError, port::publisher::PublisherSharedState,
    service::header::publish_subscribe::Header,
};
use flatbuffers::InvalidFlatbuffer;
use iceoryx2_bb_concurrency::atomic::{AtomicU64, AtomicUsize, Ordering};
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_memory::pool_allocator::ReallocGrow;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::shared_memory::*;

use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub(crate) struct SampleMutInnerSharedState<Service: crate::service::Service> {
    pub(crate) publisher_shared_state:
        Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
    pub(crate) offset_to_chunk: AtomicU64,
    pub(crate) shm_raw_ptr: AtomicUsize,
    pub(crate) slice_len: AtomicUsize,
}

unsafe impl<Service: crate::service::Service> Send for SampleMutInnerSharedState<Service> {}
impl<Service: crate::service::Service> Abandonable for SampleMutInnerSharedState<Service> {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            Service::ArcThreadSafetyPolicy::<PublisherSharedState<Service>>::abandon_in_place(
                core::ptr::NonNull::from_mut(&mut this.publisher_shared_state),
            )
        };
    }
}

impl<Service: crate::service::Service> Drop for SampleMutInnerSharedState<Service> {
    fn drop(&mut self) {
        self.publisher_shared_state
            .lock()
            .sender
            .return_loaned_sample(PointerOffset::from_value(
                self.offset_to_chunk.load(Ordering::Relaxed),
            ));
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct SampleMutSharedState<Service: crate::service::Service> {
    pub(crate) state: Service::ArcThreadSafetyPolicy<SampleMutInnerSharedState<Service>>,
}

impl<Service: crate::service::Service> SampleMutSharedState<Service> {
    pub(crate) fn new(
        publisher_shared_state: &Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
        pointer_to_chunk: ShmPointer,
        underlying_slice_len: usize,
    ) -> Self {
        Self {
            state: Service::ArcThreadSafetyPolicy::new(SampleMutInnerSharedState {
                publisher_shared_state: publisher_shared_state.clone(),
                offset_to_chunk: AtomicU64::new(pointer_to_chunk.offset.as_value()),
                shm_raw_ptr: AtomicUsize::new(pointer_to_chunk.data_ptr as usize),
                slice_len: AtomicUsize::new(underlying_slice_len),
            })
            .unwrap(),
        }
    }
}

impl<Service: crate::service::Service> ReallocGrow<ShmPointer> for SampleMutSharedState<Service> {
    unsafe fn grow(
        &self,
        ptr: ShmPointer,
        old_layout: core::alloc::Layout,
        new_layout: core::alloc::Layout,
        content_placement: iceoryx2_bb_memory::pool_allocator::ContentPlacement,
    ) -> Result<ShmPointer, AllocationGrowError> {
        let state = self.state.lock();
        let ptr = unsafe {
            state.publisher_shared_state.lock().sender.grow(
                ptr,
                old_layout,
                new_layout,
                content_placement,
            )?
        };

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

impl<Service: crate::service::Service> SampleMutSharedState<Service> {
    pub(crate) fn slice_len(&self) -> usize {
        self.state.lock().slice_len.load(Ordering::Relaxed)
    }
}

/// Acquired by a [`crate::port::publisher::Publisher`] via
///  * [`crate::port::publisher::Publisher::loan()`],
///  * [`crate::port::publisher::Publisher::loan_slice()`]
///
/// It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
pub struct SampleMut<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> {
    pub(crate) shared_state: SampleMutSharedState<Service>,
    pub(crate) chunk: ChunkMut,
    pub(crate) _payload: PhantomData<Payload>,
    pub(crate) _user_header: PhantomData<UserHeader>,
}

unsafe impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Send for SampleMut<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>: Send + Sync,
{
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> Deref for SampleMut<Service, Payload, UserHeader>
{
    type Target = Payload;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.chunk.payload_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> Deref for SampleMut<Service, [Payload], UserHeader>
{
    type Target = [Payload];
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                self.header().number_of_elements() as usize,
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> DerefMut for SampleMut<Service, Payload, UserHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> DerefMut for SampleMut<Service, [Payload], UserHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                self.header().number_of_elements() as usize,
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Debug for SampleMut<Service, Payload, UserHeader>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SampleMut<{}, {}, {}> {{ shared_state: {:?}, chunk: {:?} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<Payload>(),
            core::any::type_name::<UserHeader>(),
            self.shared_state,
            self.chunk
        )
    }
}

impl<Service: crate::service::Service, Payload: Debug, UserHeader: ZeroCopySend>
    SampleMut<Service, Flatbuffer<Payload>, UserHeader>
{
    /// Returns the serialized flatbuffer data as bytes.
    pub fn payload_bytes(&self) -> &[u8] {
        let payload_offset = self.header().payload_offset() as usize;
        let payload_ptr = self.chunk.payload_ptr();
        let payload_len = self.header().number_of_elements() as usize;

        unsafe { core::slice::from_raw_parts(payload_ptr.add(payload_offset), payload_len) }
    }

    /// Returns the root of the flatbuffer.
    pub fn payload_root<'a>(&'a self) -> Result<Payload::Inner, InvalidFlatbuffer>
    where
        Payload: flatbuffers::Follow<'a> + flatbuffers::Verifiable,
    {
        flatbuffers::root::<Payload>(self.payload_bytes())
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> SampleMut<Service, Payload, UserHeader>
{
    /// Returns a reference to the header of the sample.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan()?;
    /// println!("Sample Publisher Origin {:?}", sample.header().publisher_id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> &Header {
        unsafe { &*self.chunk.header_ptr().cast() }
    }

    /// Returns a reference to the user_header of the sample.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .user_header::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan()?;
    /// println!("Sample Publisher Origin {:?}", sample.user_header());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header(&self) -> &UserHeader {
        unsafe { &*self.chunk.user_header_ptr().cast() }
    }

    /// Returns a mutable reference to the user_header of the sample.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .user_header::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan()?;
    /// *sample.user_header_mut() = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header_mut(&mut self) -> &mut UserHeader {
        unsafe { &mut *self.chunk.user_header_mut_ptr().cast() }
    }

    /// Send a previously loaned [`crate::port::publisher::Publisher::loan_uninit()`] or
    /// [`crate::port::publisher::Publisher::loan()`] [`SampleMut`] to all connected
    /// [`crate::port::subscriber::Subscriber`]s of the service.
    ///
    /// On success the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data is returned, otherwise a [`SendError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 4567;
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn send(self) -> Result<usize, SendError> {
        let state = self.shared_state.state.lock();
        state.publisher_shared_state.lock().send_sample(
            PointerOffset::from_value(state.offset_to_chunk.load(Ordering::Relaxed)),
            self.chunk.layout().size(),
        )
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + ZeroCopySend + Debug,
    UserHeader: ZeroCopySend,
> SampleMut<Service, Payload, UserHeader>
{
    /// Returns a reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` can be packed into [`core::mem::MaybeUninit<Payload>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan()?;
    /// println!("Sample current payload {}", sample.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &Payload {
        unsafe { &*self.chunk.payload_ptr().cast() }
    }

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` can be packed into [`core::mem::MaybeUninit<Payload>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 4567;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut Payload {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + ZeroCopySend + Debug,
    UserHeader: ZeroCopySend,
> SampleMut<Service, [Payload], UserHeader>
{
    /// Returns a reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` can be packed into [`core::mem::MaybeUninit<Payload>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan_slice(1)?;
    /// println!("Sample current payload {}", sample.payload()[0]);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &[Payload] {
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                self.shared_state.slice_len(),
            )
        }
    }

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` can be packed into [`core::mem::MaybeUninit<Payload>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_slice(1)?;
    /// *sample.payload_mut()[0] = 4567;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut [Payload] {
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                self.shared_state.slice_len(),
            )
        }
    }
}
