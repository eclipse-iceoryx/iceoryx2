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

use std::{fmt::Debug, mem::MaybeUninit};

use crate::{sample_mut::SampleMut, service::header::publish_subscribe::Header};

pub struct SampleMutUninit<Service: crate::service::Service, Payload: Debug + ?Sized, UserHeader> {
    sample: SampleMut<Service, Payload, UserHeader>,
}

impl<Service: crate::service::Service, Payload: Debug + ?Sized, UserHeader>
    SampleMutUninit<Service, Payload, UserHeader>
{
    pub fn header(&self) -> &Header {
        self.sample.header()
    }

    pub fn user_header(&self) -> &UserHeader {
        self.sample.user_header()
    }

    pub fn user_header_mut(&mut self) -> &mut UserHeader {
        self.sample.user_header_mut()
    }

    pub fn payload(&self) -> &Payload {
        self.sample.payload()
    }

    pub fn payload_mut(&mut self) -> &mut Payload {
        self.sample.payload_mut()
    }
}

impl<Service: crate::service::Service, Payload: Debug, UserHeader>
    SampleMutUninit<Service, MaybeUninit<Payload>, UserHeader>
{
    /// Writes the payload to the sample and labels the sample as initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// let sample = sample.write_payload(1234);
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_payload(mut self, value: Payload) -> SampleMut<Service, Payload, UserHeader> {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    /// Extracts the value of the [`core::mem::MaybeUninit<Payload>`] container and labels the sample as initialized
    ///
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<Payload>`] really is initialized. Calling this when
    /// the content is not fully initialized causes immediate undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_uninit()?;
    /// sample.payload_mut().write(1234);
    /// let sample = unsafe { sample.assume_init() };
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(self) -> SampleMut<Service, Payload, UserHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        std::mem::transmute(self.sample)
    }
}

impl<Service: crate::service::Service, Payload: Debug, UserHeader>
    SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>
{
    /// Extracts the value of the slice of [`core::mem::MaybeUninit<Payload>`] and labels the sample as initialized
    ///
    /// # Safety
    ///
    /// The caller must ensure that every element of the slice of [`core::mem::MaybeUninit<Payload>`]
    /// is initialized. Calling this when the content is not fully initialized causes immediate undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use core::mem::MaybeUninit;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().max_slice_len(32).create()?;
    ///
    /// let slice_length = 10;
    /// let mut sample = publisher.loan_slice_uninit(slice_length)?;
    ///
    /// for element in sample.payload_mut() {
    ///     element.write(1234);
    /// }
    ///
    /// let sample = unsafe { sample.assume_init() };
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(self) -> SampleMut<Service, [Payload], UserHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        std::mem::transmute(self.sample)
    }

    /// Writes the payload to the sample and labels the sample as initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().max_slice_len(16).create()?;
    ///
    /// let slice_length = 12;
    /// let sample = publisher.loan_slice_uninit(slice_length)?;
    /// let sample = sample.write_from_fn(|n| n + 123);
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_fn<F: FnMut(usize) -> Payload>(
        mut self,
        mut initializer: F,
    ) -> SampleMut<Service, [Payload], UserHeader> {
        for (i, element) in self.payload_mut().iter_mut().enumerate() {
            element.write(initializer(i));
        }

        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }
}

impl<Service: crate::service::Service, Payload: Debug + Copy, UserHeader>
    SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>
{
    pub fn write_from_slice(
        mut self,
        value: &[Payload],
    ) -> SampleMut<Service, [Payload], UserHeader> {
        self.payload_mut()
            .copy_from_slice(unsafe { core::mem::transmute(value) });
        unsafe { self.assume_init() }
    }
}
