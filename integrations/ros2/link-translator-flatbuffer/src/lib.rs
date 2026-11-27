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
//! to.
//!
//! * The root type is the message type with dots for slashes e.g.
//!   `sensor_msgs.msg.JointState` for `sensor_msgs/msg/JointState`.
//! * A field pairs with the member of the same name. A table or struct
//!   pairs with a nested message, a vector with a sequence or an array.
//!
//! The [`FlatbufferTranslator`] names the topic type after the root type of a
//! local service. For a topic it looks the schema up by the root type in the
//! flatbuffer schema search path `iceoryx2` is configured with.
//!
//! # The Model and the Layouts
//!
//! A message is translated through a model that belongs to neither format.
//!
//! * The [`Model`](model::Model) lists the values a message holds. Each value
//!   has a kind, e.g. a scalar, a string or a sequence, and is numbered by its
//!   [`ValueIndex`](model::ValueIndex).
//! * A layout says where each value sits in one format. The [`cdr::Layout`] and
//!   the [`flatbuffer::Layout`] are derived for the same model, so value `N` is
//!   the same value in both.
//!
//! The three are compiled together from the introspection typesupport of the
//! message and the schema of the flatbuffer. For `std_msgs/msg/Header` and a
//! schema that matches it:
//!
//! ```text
//! std_msgs/msg/Header                  table Header
//! ├─ stamp                             ├─ stamp     struct Time, slot 4
//! │  ├─ sec       int32                │  ├─ sec       offset 0
//! │  └─ nanosec   uint32               │  └─ nanosec   offset 4
//! └─ frame_id     string               └─ frame_id  slot 6
//!
//!                       compile
//!                          │
//!      ┌───────────────────┼───────────────────┐
//! cdr::Layout             Model               flatbuffer::Layout
//! members                 values              fields
//! 0  Scalar(I32)  ── 0 ── Scalar(I32) ── 0 ── struct in slot 4, offset 0
//! 1  Scalar(U32)  ── 1 ── Scalar(U32) ── 1 ── struct in slot 4, offset 4
//! 2  String       ── 2 ── String      ── 2 ── table 0, slot 6
//!
//! order [0, 1, 2]                             tables  [root]
//!                                             structs [slot 4, values 0 and 1]
//! ```
//!
//! * The nested message `stamp` is not a value. Its members are values of the
//!   model, and only the flatbuffer layout records that a struct holds them.
//! * The CDR layout keeps only the order of the members. CDR writes the members
//!   of a nested message in place, one after the other, so the nesting leaves
//!   no trace in the bytes. It also has no offsets, so a member is found by
//!   reading past the ones before it.
//! * The flatbuffer layout keeps the nesting. A flatbuffer stores a nested
//!   message as a separate table or struct, and a field is reached through the
//!   slot of the table that holds it. So the layout records the tables, the
//!   structs and where each field sits in them.
//!
//! # Sequences of Messages
//!
//! A sequence or an array of messages is one value of the model, however many
//! elements it holds.
//!
//! Every element has the same shape, so the message that repeats is compiled
//! once, into a model and two layouts like any other message. The value refers
//! to them by a [`ModelIndex`](model::ModelIndex). The same index identifies
//! the model of an element and its layout in each format, so both formats agree
//! on what an element is.
//!
//! For `geometry_msgs/msg/Polygon`:
//!
//! ```text
//! geometry_msgs/msg/Polygon            table Polygon
//! └─ points   Point32[]                └─ points   [Point32], slot 4
//!    ├─ x  float32                        ├─ x  offset 0
//!    ├─ y  float32                        ├─ y  offset 4
//!    └─ z  float32                        └─ z  offset 8
//!
//! cdr::Layout                   Model                      flatbuffer::Layout
//! members                       values                     fields
//! 0  MessageSequence(0) ── 0 ── MessageSequence(0) ── 0 ── StructVector(0)
//!                                                          table 0, slot 4
//!
//! layout of model 0             model 0                    layout of model 0
//! 0  Scalar(F32)        ── 0 ── Scalar(F32)        ── 0 ── element, offset 0
//! 1  Scalar(F32)        ── 1 ── Scalar(F32)        ── 1 ── element, offset 4
//! 2  Scalar(F32)        ── 2 ── Scalar(F32)        ── 2 ── element, offset 8
//!                                                          root is a struct
//! ```
//!
//! A flatbuffer holds the elements in a vector of tables or in a vector of
//! structs. The flatbuffer layout of an element starts at that table or struct.
//! In the example the vector holds structs, so the fields of an element sit at
//! offsets from the start of its struct.
//!
//! # Translating
//!
//! Each format provides a [`MessageView`](model::MessageView) of its bytes and
//! a writer for its layout. A view hands out values by their `ValueIndex`, and
//! a writer asks a view for the values in the order its format needs them.
//!
//! ```text
//! decode  CDR        ─► cdr::Reader        ─► MessageView ─► flatbuffer::write
//! encode  flatbuffer ─► flatbuffer::Reader ─► MessageView ─► cdr::write
//! ```
//!
//! * A writer never sees the other format. It only knows its own layout and has
//!   a `MessageView` to access its values.
//! * The CDR is walked once to locate its values, since a later member starts
//!   where the earlier ones end. The flatbuffer is read in place through its
//!   offsets.
//! * The elements of a sequence of messages are again
//!   [`MessageView`](model::MessageView)s, read with the layout of the repeated
//!   message.
//!
//! # Compiling
//!
//! Compiling a message takes four steps for each of its members.
//!
//! * Pair the member with the field it goes to.
//! * Place the field in the slot of a table or at an offset of a struct.
//! * Resolve the pair into one value, a nested message or a repeated message.
//!   A pair whose types do not fit is refused.
//! * Build the result into the model and the two layouts. A nested message is
//!   compiled into the same model, a repeated message into a separate one.

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
