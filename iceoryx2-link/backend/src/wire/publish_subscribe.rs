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

use iceoryx2::service::Service;
use iceoryx2_log::{fail, origin};

use crate::service_description::SampleTypes;
use crate::wire::UnsupportedLength;
use crate::wire::sample::{
    Header, LoanError, LoanableSample, Payload, PayloadUninit, WritableSample, fits,
};

pub type Sample<S> = iceoryx2::sample::Sample<S, Payload, Header>;
pub type SampleMut<S> = iceoryx2::sample_mut::SampleMut<S, Payload, Header>;
pub type SampleMutUninit<S> =
    iceoryx2::sample_mut_uninit::SampleMutUninit<S, PayloadUninit, Header>;

pub type Publisher<S> = iceoryx2::port::publisher::Publisher<S, Payload, Header>;
pub type Subscriber<S> = iceoryx2::port::subscriber::Subscriber<S, Payload, Header>;

/// Loans an uninitialized sample of the given payload size from the
/// local publisher.
pub type LoanFn<'a, S, E> = dyn FnMut(usize) -> Result<SampleMutUninit<S>, E> + 'a;

/// Views a loaned sample's user header as writable bytes.
///
/// # Safety
///
/// `size` must be the user header size of the service the sample belongs
/// to, as its description states.
pub unsafe fn user_header_bytes_mut<S: Service>(
    sample: &mut SampleMutUninit<S>,
    size: usize,
) -> &mut [u8] {
    unsafe {
        core::slice::from_raw_parts_mut(sample.user_header_mut() as *mut Header as *mut u8, size)
    }
}

/// Views a loaned sample's payload as writable bytes.
pub fn payload_bytes_mut<S: Service>(sample: &mut SampleMutUninit<S>) -> &mut [u8] {
    let payload = sample.payload_mut();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    unsafe { core::slice::from_raw_parts_mut(payload.as_mut_ptr() as *mut u8, payload.len()) }
}

/// A sample for a given type that is not yet loaned.
pub struct UnloanedSample<'a, 'b, S: Service, E> {
    types: &'a SampleTypes,
    loan: &'a mut LoanFn<'b, S, E>,
}

impl<'a, 'b, S: Service, E> UnloanedSample<'a, 'b, S, E> {
    pub fn new(types: &'a SampleTypes, loan: &'a mut LoanFn<'b, S, E>) -> Self {
        Self { types, loan }
    }
}

impl<S: Service, E> LoanableSample for UnloanedSample<'_, '_, S, E> {
    type Sample = LoanedSample<S>;

    fn header_size(&self) -> usize {
        self.types.user_header.size
    }

    fn loan(self, payload_len: usize) -> Result<Self::Sample, LoanError> {
        let origin = origin!("UnloanedSample::loan");

        let header_size = self.types.user_header.size;
        if !fits(self.types, header_size, payload_len) {
            fail!(
                from origin,
                with LoanError::Malformed,
                "A payload of {} bytes does not fit the service description", payload_len
            );
        }
        let mut sample = fail!(
            from origin,
            when (self.loan)(payload_len),
            with LoanError::Exhausted,
            "Failed to loan a sample for a payload of {} bytes", payload_len
        );

        // TODO: Resize the loaned sample instead, once slice samples can be
        // resized through the publisher's allocation strategy.
        let payload = payload_bytes_mut(&mut sample);
        if payload.len() != payload_len {
            fail!(
                from origin,
                with LoanError::NotResizable,
                "The sample holds a payload of {} bytes, not {}", payload.len(), payload_len
            );
        }

        Ok(LoanedSample {
            header_size,
            sample,
        })
    }
}

/// A sample loaned for a given payload type.
///
/// Produced by [`LoanableSample::loan`] on an [`UnloanedSample`].
pub struct LoanedSample<S: Service> {
    header_size: usize,
    sample: SampleMutUninit<S>,
}

impl<S: Service> LoanedSample<S> {
    pub fn into_sample(self) -> SampleMutUninit<S> {
        self.sample
    }
}

impl<S: Service> WritableSample for LoanedSample<S> {
    fn payload(&mut self) -> &mut [u8] {
        payload_bytes_mut(&mut self.sample)
    }

    fn header(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength> {
        let origin = origin!("LoanedSample::header");

        if len != self.header_size {
            fail!(
                from origin,
                with UnsupportedLength,
                "A header of {} bytes does not fit the service description", len
            );
        }
        // SAFETY: the header size is the service's, as its description
        // states.
        Ok(unsafe { user_header_bytes_mut(&mut self.sample, len) })
    }
}
