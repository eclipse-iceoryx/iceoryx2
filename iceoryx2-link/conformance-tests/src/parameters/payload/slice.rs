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
use core::fmt::Debug;
use core::marker::PhantomData;

use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use super::PayloadShape;

/// A payload of `[T]` whose values are defined by `T`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SlicePayload<T>(PhantomData<T>);

/// An element type that defines the slices the suites send.
pub trait SliceElement: ZeroCopySend + TypeName + Debug + Copy + PartialEq + 'static {
    /// The slice the suites send for `n`. Different `n` give different slices.
    fn slice(n: u64) -> Vec<Self>;
}

/// Raw bytes for backends without message types. The slice is the eight
/// little endian bytes of `n`.
impl SliceElement for u8 {
    fn slice(n: u64) -> Vec<u8> {
        n.to_le_bytes().to_vec()
    }
}

impl<T: SliceElement> PayloadShape for SlicePayload<T> {
    type Type = [T];
    type Value = Vec<T>;

    fn value(n: u64) -> Vec<T> {
        T::slice(n)
    }

    fn type_detail() -> TypeDetail {
        TypeDetail::new::<T>(TypeVariant::Dynamic)
    }
}
