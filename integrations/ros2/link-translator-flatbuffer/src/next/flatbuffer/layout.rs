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

use crate::next::model::{ScalarType, ScalarValue, ValueIndex};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Layout {
    /// The root table comes first.
    pub tables: Vec<TableEntry>,
    pub fields: Vec<FieldEntry>,
}

impl Layout {
    pub fn field(&self, index: ValueIndex) -> &FieldEntry {
        &self.fields[usize::from(index.0)]
    }

    pub fn table(&self, index: TableIndex) -> &TableEntry {
        &self.tables[usize::from(index.0)]
    }
}

/// The position of a table in a [`Layout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableIndex(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableEntry {
    pub values: Vec<ValueIndex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub table: TableIndex,
    pub vtable_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldEntry {
    pub placement: FieldPlacement,
    pub kind: FieldKind,
}

/// Where a field sits in a flatbuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPlacement {
    /// In a slot of a table.
    Table(Slot),
}

impl FieldPlacement {
    /// The slot the field is found through.
    pub fn slot(self) -> Slot {
        match self {
            FieldPlacement::Table(slot) => slot,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldKind {
    Scalar {
        scalar: ScalarType,
        default: ScalarValue,
    },
    String,
}
