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

//! A neutral model for describing the contents of a message.
//!
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
pub mod flatbuffer;
pub mod model;

pub use compile::{CompileError, Compiled, Refusal, compile};
