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

use core::error::Error;
use core::fmt::{Display, Formatter};

use super::{Shape, Translator};
use crate::Never;

/// The translator of a shape the middleware does not carry.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnsupportedTranslator;

/// The middleware does not carry the shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedShape;

impl Display for UnsupportedShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnsupportedShape")
    }
}

impl Error for UnsupportedShape {}

impl<S: Shape> Translator<S> for UnsupportedTranslator {
    type RemoteTypes = Never;
    type Transcoders = Never;
    type Error = UnsupportedShape;

    fn local(&self, remote: &Never) -> Result<S::LocalTypes, UnsupportedShape> {
        match *remote {}
    }

    fn remote(&self, _: &S::LocalTypes) -> Result<Never, UnsupportedShape> {
        Err(UnsupportedShape)
    }

    fn transcoders(&self, _: &S::LocalTypes, remote: &Never) -> Result<Never, UnsupportedShape> {
        match *remote {}
    }
}
