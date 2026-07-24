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

use core::{
    fmt::{Debug, Display},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatbufferError<T: Debug + Display + Clone + PartialEq + Eq>(T);

impl<T: Debug + Display + Clone + PartialEq + Eq> Deref for FlatbufferError<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Debug + Display + Clone + PartialEq + Eq> Display for FlatbufferError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Debug + Display + Clone + PartialEq + Eq> From<T> for FlatbufferError<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Debug + Display + Clone + PartialEq + Eq> core::error::Error for FlatbufferError<T> {}
