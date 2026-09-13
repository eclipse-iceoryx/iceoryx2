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

mod fixed_size;
pub(crate) mod slice;

pub use fixed_size::FixedSizePayload;
pub use slice::SlicePayload;

use core::fmt::Debug;

use iceoryx2::service::static_config::message_type_details::TypeDetail;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

/// The shape of a payload the suites carry, a type a service is created
/// with and an owned value of it.
pub trait PayloadShape: 'static {
    /// The payload type a service is created with, `T` or `[T]`.
    type Type: IceoryxSend + ZeroCopySend + Debug + ?Sized + 'static;
    /// A value of it in the suites' hands.
    type Value: Clone + PartialEq + Debug;

    /// The value the suites send for `n`, distinct per `n`.
    fn value(n: u64) -> Self::Value;

    /// The type detail a service of this payload carries.
    fn type_detail() -> TypeDetail;
}
