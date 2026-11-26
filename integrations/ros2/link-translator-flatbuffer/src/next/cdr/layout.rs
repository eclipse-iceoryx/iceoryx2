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

use crate::next::model::{ScalarType, ValueIndex};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Layout {
    /// By value index.
    pub members: Vec<MemberEntry>,
    /// The values in the order the CDR holds them.
    pub order: Vec<ValueIndex>,
}

impl Layout {
    pub fn member(&self, index: ValueIndex) -> &MemberEntry {
        &self.members[usize::from(index.0)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberEntry {
    pub kind: MemberKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberKind {
    Scalar(ScalarType),
    String,
}
