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

use core::mem::MaybeUninit;

use iceoryx2::service::Service;
use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::TypeVariant;

use crate::service_description::PublishSubscribeTypes;

/// The untyped user header as it passes through a relay.
pub type Header = CustomHeaderMarker;
/// The untyped payload as it passes through a relay.
pub type Payload = [CustomPayloadMarker];
pub type PayloadUninit = [MaybeUninit<CustomPayloadMarker>];

pub type Sample<S> = iceoryx2::sample::Sample<S, Payload, Header>;
pub type SampleMut<S> = iceoryx2::sample_mut::SampleMut<S, Payload, Header>;
pub type SampleMutUninit<S> =
    iceoryx2::sample_mut_uninit::SampleMutUninit<S, PayloadUninit, Header>;

pub type Publisher<S> = iceoryx2::port::publisher::Publisher<S, Payload, Header>;
pub type Subscriber<S> = iceoryx2::port::subscriber::Subscriber<S, Payload, Header>;

/// Loans an uninitialized sample of the given payload size from the
/// local publisher, so a relay can receive directly into shared memory.
pub type LoanFn<'a, S, LoanError> = dyn FnMut(usize) -> Result<SampleMutUninit<S>, LoanError> + 'a;

/// Views a sample's user header as bytes.
///
/// # Safety
///
/// `size` must be the user header size of the service the sample belongs
/// to, as its description states.
pub unsafe fn user_header_bytes(user_header: &Header, size: usize) -> &[u8] {
    unsafe { core::slice::from_raw_parts(user_header as *const Header as *const u8, size) }
}

/// Views a sample's payload as bytes.
pub fn payload_bytes(payload: &Payload) -> &[u8] {
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
}

/// Whether a header and a payload of these lengths fit a sample of the
/// described service, the precondition of [`initialize_sample`] and of
/// writing into [`regions_mut`].
pub fn fits(types: &PublishSubscribeTypes, header: usize, payload: usize) -> bool {
    if header != types.user_header.size {
        return false;
    }
    let size = types.payload.size;
    match types.payload.variant {
        TypeVariant::FixedSize => payload == size,
        TypeVariant::Dynamic => size > 0 && payload.is_multiple_of(size),
    }
}

/// The user header and the payload of a loaned sample as writable bytes.
///
/// # Safety
///
/// `header` must be the user header size of the service the sample
/// belongs to, as its description states.
pub unsafe fn regions_mut<S: Service>(
    sample: &mut SampleMutUninit<S>,
    header_size: usize,
) -> (&mut [u8], &mut [u8]) {
    unsafe {
        let header = core::slice::from_raw_parts_mut(
            sample.user_header_mut() as *mut Header as *mut u8,
            header_size,
        );
        let payload = sample.payload_mut();
        let payload =
            core::slice::from_raw_parts_mut(payload.as_mut_ptr() as *mut u8, payload.len());
        (header, payload)
    }
}

/// Initializes a loaned sample from user header and payload bytes.
///
/// # Safety
///
/// `user_header` must be exactly the user header size of the service the
/// sample belongs to, and `payload` must not exceed the sample's payload.
pub unsafe fn initialize_sample<S: Service>(
    mut sample: SampleMutUninit<S>,
    user_header: &[u8],
    payload: &[u8],
) -> SampleMut<S> {
    debug_assert!(payload.len() <= sample.payload_mut().len());
    unsafe {
        core::ptr::copy_nonoverlapping(
            user_header.as_ptr(),
            sample.user_header_mut() as *mut Header as *mut u8,
            user_header.len(),
        );
        core::ptr::copy_nonoverlapping(
            payload.as_ptr(),
            sample.payload_mut().as_mut_ptr() as *mut u8,
            payload.len(),
        );
        sample.assume_init()
    }
}
