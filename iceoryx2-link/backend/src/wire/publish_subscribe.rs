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

//! Implements the loan state machine of [`crate::wire::sample`] for the
//! samples of a publish-subscribe service by implementing
//! [`UninitializedSample`] for them.

use iceoryx2::service::Service;
use iceoryx2_log::{fail, origin};

use crate::wire::region::ResizeError;
use crate::wire::sample::{FlatbufferMemory, Header, Payload, PayloadUninit, UninitializedSample};

pub type Sample<S> = iceoryx2::sample::Sample<S, Payload, Header>;
pub type SampleMut<S> = iceoryx2::sample_mut::SampleMut<S, Payload, Header>;
pub type SampleMutUninit<S> =
    iceoryx2::sample_mut_uninit::SampleMutUninit<S, PayloadUninit, Header>;

pub type Publisher<S> = iceoryx2::port::publisher::Publisher<S, Payload, Header>;
pub type Subscriber<S> = iceoryx2::port::subscriber::Subscriber<S, Payload, Header>;

/// View of the initialized bytes of a sample's payload.
pub fn payload_bytes<S: Service>(sample: &Sample<S>) -> &[u8] {
    let payload = sample.payload();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    let bytes =
        unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) };
    &bytes[sample.header().payload_offset() as usize..]
}

impl<S: Service> UninitializedSample for SampleMutUninit<S> {
    type InitializedSample = SampleMut<S>;
    type FlatbufferMemory = iceoryx2::sample_mut_uninit::FlatbufferMemory<S>;

    fn user_header(&self) -> &Header {
        SampleMutUninit::user_header(self)
    }

    fn user_header_mut(&mut self) -> &mut Header {
        SampleMutUninit::user_header_mut(self)
    }

    fn payload(&self) -> &PayloadUninit {
        SampleMutUninit::payload(self)
    }

    fn payload_mut(&mut self) -> &mut PayloadUninit {
        SampleMutUninit::payload_mut(self)
    }

    fn flatbuffer_memory(&self) -> Self::FlatbufferMemory {
        self.__internal_create_resizable_memory()
    }

    fn finish_serialized(&mut self, payload: *const u8) {
        self.__internal_finish_serialized(payload)
    }

    unsafe fn assume_init(self) -> SampleMut<S> {
        // SAFETY: the caller wrote the header and the payload.
        unsafe { SampleMutUninit::assume_init(self) }
    }
}

impl<S: Service> FlatbufferMemory for iceoryx2::sample_mut_uninit::FlatbufferMemory<S> {
    fn grow(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("FlatbufferMemory::grow");

        fail!(
            from origin,
            when self.grow_downwards_with_size(len, 0),
            with ResizeError::Exhausted,
            "Failed to grow the memory to {} bytes", len
        );
        Ok(())
    }
}
