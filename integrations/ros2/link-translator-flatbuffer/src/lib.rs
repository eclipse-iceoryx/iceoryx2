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

//! A translator between flatbuffer payloads and ROS 2 messages.
//!
//! The translator reads the schema of the payload and the introspection
//! typesupport of the message. A schema describes the message it translates
//! to. For example, `sensor_msgs/msg/JointState` is described by:
//!
//! ```text
//! namespace sensor_msgs.msg;
//!
//! table JointState {
//!     header: std_msgs.msg.Header;
//!     name: [string];
//!     position: [double];
//!     velocity: [double];
//!     effort: [double];
//! }
//!
//! root_type JointState;
//! ```
//!
//! * The root type is the message type with dots for slashes.
//! * A field pairs with the member of the same name. A table or struct
//!   pairs with a nested message, a vector with a sequence or an array.
//!
//! The [`FlatbufferTranslator`] names the topic type after a local
//! service's root type. For a topic it looks the schema up in the
//! flatbuffer schema search path `iceoryx2` is configured with, by the
//! root type, as `iceoryx2` does for a service.
//!
//! The translation goes through a neutral model for describing the contents
//! of a message.
//! The [`Model`](model::Model) is used to derive the layouts for different message formats
//! independently. The model describes the contents by [`Kind`](model::Kind), which are
//! mapped to different layouts by their [`ValueIndex`](model::ValueIndex).
//!
//! ```text
//! model           layout of one format      layout of another format
//! 0 String        value 0, first            value 0, in slot 6
//! 1 Scalar U32    value 1, second           value 1, in slot 8
//! 2 Scalar U32    value 2, third            value 2, in slot 4
//! ```

pub mod cdr;
mod compile;
mod decode;
mod encode;
pub mod flatbuffer;
pub mod model;
mod transcoder;
mod translator;
mod validation;

pub use compile::{CompileError, Compiled, Refusal, compile};
pub use decode::{DecodeError, decode};
pub use encode::{EncodeError, encode};
pub use transcoder::{FlatbufferTranscoder, TranscodeFailure};
pub use translator::{FlatbufferTranslator, TranslationError};
pub use validation::{TypeMismatch, type_mismatches};
