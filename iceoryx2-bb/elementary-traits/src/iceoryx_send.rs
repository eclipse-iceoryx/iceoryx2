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

use crate::type_name::TypeName;

#[doc(hidden)]
/// Trait to prevent the IceoryxSend trait from being implemented outside the crate.
///
/// # Safety
///
/// * Internal marker trait, this shall be NEVER implemented by the user!
///
pub unsafe trait __InternalNoTouchyFishy {}

/// Marker trait that identifies types that can be transmitted via iceoryx2.
#[allow(private_bounds)]
pub trait IceoryxSend: __InternalNoTouchyFishy + TypeName {}

/// ``` compile_fail
/// use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
///
/// struct Foo(u32);
/// unsafe impl IceoryxSend for Foo {};
/// ```
#[cfg(doctest)]
fn iceoryx_send_cannot_be_implemented() {}
