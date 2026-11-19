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
use iceoryx2_bb_container::string::String as _;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::iceoryx_send::{__InternalNoTouchyFishy, IceoryxSend};
use iceoryx2_bb_elementary_traits::type_name::TypeName;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::service::static_config::message_type_details::TypeName as TypeNameString;

#[repr(C)]
#[derive(Debug, ZeroCopySend, Clone, Default)]
#[doc(hidden)]
pub struct CustomHeaderMarker {}

#[repr(C)]
#[derive(Debug, ZeroCopySend, Clone)]
#[doc(hidden)]
pub struct CustomPayloadMarker(u8);

#[repr(C)]
#[derive(ZeroCopySend, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub struct CustomKeyMarker(u8);

/// Marker Type to mark a payload as serialized via Flatbuffer.
#[repr(C)]
pub struct Flatbuffer<T> {
    _data: u8,
    _phantom: PhantomData<T>,
}

impl<T> Debug for Flatbuffer<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Flatbuffer<{}>", core::any::type_name::<T>())
    }
}

unsafe impl<T> __InternalNoTouchyFishy for Flatbuffer<T> {}
impl<T> IceoryxSend for Flatbuffer<T> {}

/// The type name of the `Flatbuffer` marker.
///
/// A service records it with the root type of the payload's schema appended
/// in angle brackets, e.g. `iox2::Flatbuffer<Testing.UnboundedData>`.
pub(crate) const FLATBUFFER_TYPE_NAME: &str = "iox2::Flatbuffer";

unsafe impl<T> TypeName for Flatbuffer<T> {
    unsafe fn type_name() -> &'static str {
        FLATBUFFER_TYPE_NAME
    }
}

/// The type name a service records for a flatbuffer payload whose schema
/// declares `root_type`, or `None` when it does not fit.
pub fn flatbuffer_type_name(root_type: &str) -> Option<TypeNameString> {
    let mut type_name = TypeNameString::new();
    for piece in [FLATBUFFER_TYPE_NAME, "<", root_type, ">"] {
        type_name.push_bytes(piece.as_bytes()).ok()?;
    }
    Some(type_name)
}

/// Whether `type_name` is the recorded type name of a flatbuffer payload.
pub fn is_flatbuffer_type_name(type_name: &str) -> bool {
    match type_name.strip_prefix(FLATBUFFER_TYPE_NAME) {
        Some("") => true,
        Some(composed) => composed.starts_with('<') && composed.ends_with('>'),
        None => false,
    }
}
