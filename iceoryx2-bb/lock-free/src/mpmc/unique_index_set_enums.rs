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

use iceoryx2_bb_elementary::enum_gen;

/// Describes if indices can still be acquired after the call to
/// [`UniqueIndexSet::release_raw_index()`](crate::mpmc::unique_index_set::UniqueIndexSet::release_raw_index()).
/// or
/// [`RobustUniqueIndexSet::release()`](crate::mpmc::robust_unique_index_set::RobustUniqueIndexSet::release()).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ReleaseMode {
    /// No more indices can be acquired with [`UniqueIndexSet::acquire_raw_index()`](crate::mpmc::unique_index_set::UniqueIndexSet::acquire_raw_index()) or
    /// [`RobustUniqueIndexSet::acquire()`](crate::mpmc::robust_unique_index_set::RobustUniqueIndexSet::acquire()) if the
    /// released index was the last one.
    LockIfLastIndex,
    /// Indices can still be acquired with [`UniqueIndexSet::acquire_raw_index()`](crate::mpmc::unique_index_set::UniqueIndexSet::acquire_raw_index()) or
    /// [`RobustUniqueIndexSet::acquire()`](crate::mpmc::robust_unique_index_set::RobustUniqueIndexSet::acquire())
    /// after the
    /// operation
    #[default]
    Default,
}

/// Defines the state of the [`UniqueIndexSet`](crate::mpmc::unique_index_set::UniqueIndexSet) or
/// [`RobustUniqueIndexSet`](crate::mpmc::robust_unique_index_set::RobustUniqueIndexSet) after the release operation
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReleaseState {
    /// The unique index set is in locked mode since the last index was released. New indices
    /// can no longer acquired from the unique index set.
    Locked,
    /// New indices can still be acquired from the unique index set
    Unlocked,
}

/// It states the reason if an index could not be acquired.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UniqueIndexSetAcquireFailure {
    /// The unique index set does not contain any more indices
    OutOfIndices,
    /// The unique index set is in a locked state and indices can no longer be acquired.
    IsLocked,
}

enum_gen! { UniqueIndexCreationError
  entry:
    ProvidedCapacityGreaterThanMaxCapacity,
    ProvidedCapacityIsZero
}
