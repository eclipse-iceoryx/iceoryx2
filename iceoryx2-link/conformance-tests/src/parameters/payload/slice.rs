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

/// A slice payload of `T` elements, one value the eight bytes of a
/// number.
#[derive(Debug, Default, Clone, Copy)]
pub struct SlicePayload<T>(PhantomData<T>);

/// The elements a slice value holds, the bytes of a `u64`.
pub(crate) const SLICE_LEN: usize = size_of::<u64>();

impl<T> PayloadShape for SlicePayload<T>
where
    T: ZeroCopySend + TypeName + Debug + Copy + PartialEq + From<u8> + 'static,
{
    type Type = [T];
    type Value = Vec<T>;

    fn value(n: u64) -> Vec<T> {
        n.to_le_bytes().into_iter().map(T::from).collect()
    }

    fn type_detail() -> TypeDetail {
        TypeDetail::new::<T>(TypeVariant::Dynamic)
    }
}
