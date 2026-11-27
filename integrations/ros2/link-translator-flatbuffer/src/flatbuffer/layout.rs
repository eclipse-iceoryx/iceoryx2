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
    /// By table index.
    ///
    /// The root table comes first and a nested table comes after the table
    /// that holds it.
    pub tables: Vec<TableEntry>,
    /// By struct index.
    pub structs: Vec<StructEntry>,
    /// By value index.
    pub fields: Vec<FieldEntry>,
    /// By model index.
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

    /// Adds a table at `placement`.
    ///
    /// Returns the index of the table.
    pub(crate) fn add_table(&mut self, placement: TablePlacement) -> TableIndex {
        self.tables.push(TableEntry {
            placement,
            values: Vec::new(),
        });

        TableIndex((self.tables.len() - 1) as u16)
    }

    /// Adds the value at `index` to the table that holds `slot`.
    pub(crate) fn add_table_value(&mut self, slot: Slot, index: ValueIndex) {
        self.tables[usize::from(slot.table.0)].values.push(index);
    }

    /// Adds the struct of `number_of_bytes` and `alignment` in `slot`.
    pub(crate) fn add_struct(&mut self, slot: Slot, number_of_bytes: u16, alignment: u16) {
        self.structs.push(StructEntry {
            slot,
            number_of_bytes,
            alignment,
            values: Vec::new(),
        });
    }

    /// Adds the value at `index` to the struct that `placement` places it in.
    pub(crate) fn add_struct_value(&mut self, placement: FieldPlacement, index: ValueIndex) {
        let values = match placement {
            // The struct sits in a slot of a table.
            FieldPlacement::Struct { slot, .. } => {
                let entry = self
                    .structs
                    .iter_mut()
                    .find(|entry| entry.slot == slot)
                    .expect("the struct of a placement is part of the layout");
                &mut entry.values
            }
            // The struct is the root of the layout.
            FieldPlacement::VectorElement { .. } => {
                let RootObject::Struct { values, .. } = &mut self.root else {
                    unreachable!("the root of the layout is a struct")
                };
                values
            }
            FieldPlacement::Table(_) => unreachable!("the value is placed in a struct"),
        };
        values.push(index);
    }

    /// Orders the values of each struct by their offsets.
    pub(crate) fn sort_struct_values(&mut self) {
        for entry in &mut self.structs {
            sort_by_offset(&mut entry.values, &self.fields);
        }
        if let RootObject::Struct { values, .. } = &mut self.root {
            sort_by_offset(values, &self.fields);
        }
    }
}

fn sort_by_offset(values: &mut [ValueIndex], fields: &[FieldEntry]) {
    values.sort_by_key(|index| match fields[usize::from(index.0)].placement {
        FieldPlacement::Struct { offset, .. } | FieldPlacement::VectorElement { offset } => offset,
        FieldPlacement::Table(_) => unreachable!("the value sits in a struct"),
    });
}

/// The table or the struct that holds one message in a flatbuffer.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum RootObject {
    #[default]
    Table,
    Struct {
        number_of_bytes: u16,
        alignment: u16,
        /// The values stored in the struct in the order of their offsets, by
        /// value index in the [`Layout`].
        values: Vec<ValueIndex>,
    },
}

/// The position of a table in a [`Layout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableIndex(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableEntry {
    pub placement: TablePlacement,
    /// The values stored in the table, by value index in the [`Layout`].
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

/// The number of bytes from the start of a struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructOffset(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructEntry {
    pub slot: Slot,
    pub number_of_bytes: u16,
    pub alignment: u16,
    /// The values stored in the struct in the order of their offsets, by
    /// value index in the [`Layout`].
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
    Struct { slot: Slot, offset: StructOffset },
    /// Within a struct that is an element of a vector.
    VectorElement { offset: StructOffset },
}

impl FieldPlacement {
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
