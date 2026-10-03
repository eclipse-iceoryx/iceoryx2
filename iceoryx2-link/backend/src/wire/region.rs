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

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

/// Why a region was not resized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeError {
    /// The region cannot hold a payload of this length.
    UnsupportedLength,
    /// The region keeps the length it has.
    NotResizable,
    /// The region has no memory left for the payload.
    Exhausted,
    /// The region cannot be written from this end.
    DirectionUnsupported,
}

impl core::fmt::Display for ResizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResizeError::{self:?}")
    }
}

impl core::error::Error for ResizeError {}

/// A writable byte region whose length is set by its writer.
pub trait Region {
    type ForwardRegion<'a>: ForwardRegion
    where
        Self: 'a;
    type BackwardRegion<'a>: BackwardRegion
    where
        Self: 'a;

    /// Fills the region from its front.
    fn forward(&mut self) -> Self::ForwardRegion<'_>;

    /// Fills the region from its end.
    fn backward(&mut self) -> Self::BackwardRegion<'_>;
}

/// A region written from its front.
pub trait ForwardRegion: DerefMut<Target = [u8]> {
    /// Resizes the region to `len` bytes, keeping its bytes at the front.
    ///
    /// Fails when the region cannot hold `len` bytes.
    fn resize(&mut self, len: usize) -> Result<(), ResizeError>;
}

/// A region written from its end.
pub trait BackwardRegion: DerefMut<Target = [u8]> {
    /// Resizes the region to `len` bytes, keeping its bytes at the end.
    ///
    /// Fails when the region cannot hold `len` bytes.
    fn resize(&mut self, len: usize) -> Result<(), ResizeError>;
}

impl<R: Region + ?Sized> Region for &mut R {
    type ForwardRegion<'a>
        = R::ForwardRegion<'a>
    where
        Self: 'a;
    type BackwardRegion<'a>
        = R::BackwardRegion<'a>
    where
        Self: 'a;

    fn forward(&mut self) -> Self::ForwardRegion<'_> {
        (**self).forward()
    }

    fn backward(&mut self) -> Self::BackwardRegion<'_> {
        (**self).backward()
    }
}

/// A `Vec<u8>` written from its front.
pub struct VecForward<'a>(&'a mut Vec<u8>);

impl Deref for VecForward<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}

impl DerefMut for VecForward<'_> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.0
    }
}

impl ForwardRegion for VecForward<'_> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        self.0.resize(len, 0);
        Ok(())
    }
}

/// A `Vec<u8>` written from its end.
pub struct VecBackward<'a>(&'a mut Vec<u8>);

impl Deref for VecBackward<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}

impl DerefMut for VecBackward<'_> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.0
    }
}

impl BackwardRegion for VecBackward<'_> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        let current = self.0.len();
        if len > current {
            self.0.resize(len, 0);
            self.0.rotate_right(len - current);
        } else {
            self.0.drain(..current - len);
        }
        Ok(())
    }
}

impl Region for Vec<u8> {
    type ForwardRegion<'a> = VecForward<'a>;
    type BackwardRegion<'a> = VecBackward<'a>;

    fn forward(&mut self) -> Self::ForwardRegion<'_> {
        VecForward(self)
    }

    fn backward(&mut self) -> Self::BackwardRegion<'_> {
        VecBackward(self)
    }
}

impl ForwardRegion for &mut [u8] {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        match len == self.len() {
            true => Ok(()),
            false => Err(ResizeError::UnsupportedLength),
        }
    }
}

impl BackwardRegion for &mut [u8] {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        match len == self.len() {
            true => Ok(()),
            false => Err(ResizeError::UnsupportedLength),
        }
    }
}

impl Region for [u8] {
    type ForwardRegion<'a> = &'a mut [u8];
    type BackwardRegion<'a> = &'a mut [u8];

    fn forward(&mut self) -> Self::ForwardRegion<'_> {
        self
    }

    fn backward(&mut self) -> Self::BackwardRegion<'_> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn forward_vec_keeps_its_bytes_at_the_front() {
        const WRITTEN: &[u8] = b"abcde";
        const GROWN: usize = 8;
        const SHRUNK: usize = 3;

        let mut vec = Vec::new();
        let mut region = vec.forward();
        region
            .resize(WRITTEN.len())
            .expect("vec supports any length");
        region.copy_from_slice(WRITTEN);

        region.resize(GROWN).expect("vec supports any length");
        assert_that!(&region[..WRITTEN.len()], eq WRITTEN);
        assert_that!(&region[WRITTEN.len()..], eq & [0; GROWN - WRITTEN.len()]);

        region.resize(SHRUNK).expect("vec supports any length");
        assert_that!(&*region, eq & WRITTEN[..SHRUNK]);

        assert_that!(vec, eq WRITTEN[..SHRUNK].to_vec());
    }

    #[test]
    fn backward_vec_keeps_its_bytes_at_the_end() {
        const WRITTEN: &[u8] = b"wxyz";
        const GROWN: usize = 8;
        const SHRUNK: usize = 3;

        let mut vec = Vec::new();
        let mut region = vec.backward();
        region
            .resize(WRITTEN.len())
            .expect("vec supports any length");
        region.copy_from_slice(WRITTEN);

        region.resize(GROWN).expect("vec supports any length");
        assert_that!(
            &region[..GROWN - WRITTEN.len()],
            eq & [0; GROWN - WRITTEN.len()]
        );
        assert_that!(&region[GROWN - WRITTEN.len()..], eq WRITTEN);

        region.resize(SHRUNK).expect("vec supports any length");
        assert_that!(&*region, eq & WRITTEN[WRITTEN.len() - SHRUNK..]);

        assert_that!(vec, eq WRITTEN[WRITTEN.len() - SHRUNK..].to_vec());
    }

    #[test]
    fn fixed_slice_only_resizes_to_its_own_length() {
        const LENGTH: usize = 4;

        let mut bytes = [0u8; LENGTH];

        let mut forward = bytes.forward();
        assert_that!(ForwardRegion::resize(&mut forward, LENGTH), is_ok);
        assert_that!(ForwardRegion::resize(&mut forward, LENGTH + 1), eq Err(ResizeError::UnsupportedLength));
        assert_that!(ForwardRegion::resize(&mut forward, LENGTH - 1), eq Err(ResizeError::UnsupportedLength));

        let mut backward = bytes.backward();
        assert_that!(BackwardRegion::resize(&mut backward, LENGTH), is_ok);
        assert_that!(BackwardRegion::resize(&mut backward, LENGTH + 1), eq Err(ResizeError::UnsupportedLength));
        assert_that!(BackwardRegion::resize(&mut backward, LENGTH - 1), eq Err(ResizeError::UnsupportedLength));
    }
}
