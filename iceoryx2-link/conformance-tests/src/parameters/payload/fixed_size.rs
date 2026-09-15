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
use core::fmt::Debug;
use core::marker::PhantomData;

use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use super::PayloadShape;

/// A fixed-size payload of `T`.
#[derive(Debug, Default, Clone, Copy)]
pub struct FixedSizePayload<T>(PhantomData<T>);

impl<T> PayloadShape for FixedSizePayload<T>
where
    T: IceoryxSend + ZeroCopySend + TypeName + Debug + Copy + PartialEq + From<u64> + 'static,
{
    type Type = T;
    type Value = T;

    fn value(n: u64) -> T {
        T::from(n)
    }

    fn type_detail() -> TypeDetail {
        TypeDetail::new::<T>(TypeVariant::FixedSize)
    }
}
