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

use crate::model::{ModelIndex, ScalarType, ScalarValue, ValueIndex};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Layout {
    pub root: RootObject,
    /// The root table comes first and a nested table comes after the table
    /// that holds it.
    pub tables: Vec<TableEntry>,
    pub structs: Vec<StructEntry>,
    pub fields: Vec<FieldEntry>,
    /// The layouts of the messages that repeat, by model index.
    pub repeated: Vec<Layout>,
}

impl Layout {
    pub fn field(&self, index: ValueIndex) -> &FieldEntry {
        &self.fields[usize::from(index.0)]
    }

    pub fn table(&self, index: TableIndex) -> &TableEntry {
        &self.tables[usize::from(index.0)]
    }

    pub fn repeated(&self, index: ModelIndex) -> &Layout {
        &self.repeated[usize::from(index.0)]
    }
}

/// The table or the struct that holds one message in a flatbuffer.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum RootObject {
    #[default]
    Table,
    Struct {
        number_of_bytes: u16,
        alignment: u16,
        /// The values the struct holds, in the order of their offsets.
        values: Vec<ValueIndex>,
    },
}

/// The position of a table in a [`Layout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableIndex(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableEntry {
    pub placement: TablePlacement,
    /// The values the table holds outside of its structs.
    pub values: Vec<ValueIndex>,
}

/// Where a table sits in a flatbuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TablePlacement {
    Root,
    /// In the slot of another table.
    Nested(Slot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub table: TableIndex,
    pub vtable_offset: u16,
}

/// The position of a struct in a [`Layout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructIndex(pub u16);

/// A number of bytes from the start of a struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructOffset(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructEntry {
    pub slot: Slot,
    pub number_of_bytes: u16,
    pub alignment: u16,
    /// The values the struct holds, in the order of their offsets.
    pub values: Vec<ValueIndex>,
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
    /// Within the struct that a slot holds.
    Struct {
        slot: Slot,
        /// From the start of the struct to the field.
        offset: StructOffset,
    },
    /// Within a struct that is an element of a vector.
    VectorElement {
        /// From the start of the struct to the field.
        offset: StructOffset,
    },
}

impl FieldPlacement {
    /// The slot the field is found through.
    ///
    /// Returns None for a field in an element of a vector.
    pub fn slot(self) -> Option<Slot> {
        match self {
            FieldPlacement::Table(slot) | FieldPlacement::Struct { slot, .. } => Some(slot),
            FieldPlacement::VectorElement { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldKind {
    Scalar {
        scalar_type: ScalarType,
        default: ScalarValue,
    },
    String,
    ScalarVector(ScalarType),
    StringVector,
    TableVector(ModelIndex),
    StructVector(ModelIndex),
    /// Within a struct.
    ScalarArray {
        scalar_type: ScalarType,
        number_of_elements: u32,
    },
}
