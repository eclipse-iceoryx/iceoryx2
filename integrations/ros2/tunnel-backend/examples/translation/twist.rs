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

//! Hand-written shared-memory mirror of `geometry_msgs/msg/Twist` and the
//! [`Translator`] bridging it to the CDR wire. The reference for the
//! translator contract; later generated from the `.msg` definition.

#![allow(dead_code)]

use core::alloc::Layout;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_tunnel_backend::TopicDescription;
use iceoryx2_services_tunnel_backend::traits::{
    PayloadLayout, ResizableBuffer, TranslationMode, Translator,
};
use iceoryx2_services_tunnel_backend::types::service_description::ServiceDescription;
use serde::{Deserialize, Serialize};

pub const TWIST_TYPE_NAME: &str = "geometry_msgs/msg/Twist";
/// 4-byte CDR encapsulation header + six f64s.
const TWIST_WIRE_SIZE: usize = 4 + core::mem::size_of::<Twist>();

#[derive(Debug, Default, Clone, Copy, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[type_name("geometry_msgs/msg/Vector3")]
#[repr(C)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[type_name("geometry_msgs/msg/Twist")]
#[repr(C)]
pub struct Twist {
    pub linear: Vector3,
    pub angular: Vector3,
}

#[derive(Debug)]
pub enum TranslationError {
    UnexpectedPayloadSize,
    Serialize(cdr::Error),
    Deserialize(cdr::Error),
}

impl core::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranslationError::{self:?}")
    }
}

impl core::error::Error for TranslationError {}

/// Translates [`Twist`] payloads between the native struct and its CDR wire
/// representation. Every other type passes through unchanged.
#[derive(Debug, Default)]
pub struct TwistTranslator;

impl Translator for TwistTranslator {
    type Error = TranslationError;
    type EndpointDescription = TopicDescription;

    fn resolve(
        &self,
        _service: &ServiceDescription,
        endpoint: &TopicDescription,
    ) -> Result<TranslationMode, Self::Error> {
        // A multi-type translator matches on the type name here — one arm
        // per translated type, each answering with its native layout — and
        // dispatches on the same name in to_wire/from_wire.
        if endpoint.type_name.as_str() == TWIST_TYPE_NAME {
            Ok(TranslationMode::Translate {
                payload_layout: PayloadLayout::FixedSize(Layout::new::<Twist>()),
            })
        } else {
            Ok(TranslationMode::Passthrough)
        }
    }

    fn to_wire(
        &self,
        _service: &ServiceDescription,
        _endpoint: &TopicDescription,
        payload: &[u8],
        wire: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        if payload.len() != core::mem::size_of::<Twist>() {
            return Err(TranslationError::UnexpectedPayloadSize);
        }
        // SAFETY: the payload is a Twist — resolve() only selects services
        // carrying the Twist type name, and the type-name contract ties the
        // name to this repr(C) layout.
        let twist = unsafe { payload.as_ptr().cast::<Twist>().read_unaligned() };

        let region = wire.resize(TWIST_WIRE_SIZE);
        let mut cursor = &mut region[..TWIST_WIRE_SIZE];
        cdr::serialize_into::<_, _, _, cdr::CdrLe>(&mut cursor, &twist, cdr::Infinite)
            .map_err(TranslationError::Serialize)?;

        Ok(TWIST_WIRE_SIZE)
    }

    fn from_wire(
        &self,
        _service: &ServiceDescription,
        _endpoint: &TopicDescription,
        wire: &[u8],
        payload: &mut impl ResizableBuffer,
    ) -> Result<usize, Self::Error> {
        let twist: Twist = cdr::deserialize(wire).map_err(TranslationError::Deserialize)?;

        // SAFETY: Twist is repr(C) with all-f64 fields — no padding, every
        // byte initialized.
        let bytes = unsafe {
            core::slice::from_raw_parts(
                (&twist as *const Twist).cast::<u8>(),
                core::mem::size_of::<Twist>(),
            )
        };
        let region = payload.resize(bytes.len());
        region[..bytes.len()].copy_from_slice(bytes);

        Ok(bytes.len())
    }
}
