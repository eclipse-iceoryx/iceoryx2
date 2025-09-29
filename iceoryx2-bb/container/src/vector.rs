// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

/// Defines the interface of a vector.
pub trait Vector<T> {
    /// Returns a mutable slice to the contents of the vector
    fn as_mut_slice(&mut self) -> &mut [T];

    /// Returns a slice to the contents of the vector
    fn as_slice(&self) -> &[T];

    /// Returns the capacity of the vector
    fn capacity(&self) -> usize;

    /// Removes all elements from the vector
    fn clear(&mut self);

    /// Append all elements from other via [`Clone`].
    fn extend_from_slice(&mut self, other: &[T]) -> bool
    where
        T: Clone;

    /// Inserts an element at the provided index and shifting all elements
    /// after the index to the right.
    fn insert(&mut self, index: usize, element: T) -> bool;

    /// Returns true if the vector is empty, otherwise false
    fn is_empty(&self) -> bool;

    /// Returns true if the vector is full, otherwise false
    fn is_full(&self) -> bool;

    /// Returns the number of elements stored inside the vector
    fn len(&self) -> usize;

    /// Removes the last element of the vector and returns it to the user. If the vector is empty
    /// it returns [`None`].
    fn pop(&mut self) -> Option<T>;

    /// Adds an element at the end of the vector. If the vector is full and the element cannot be
    /// added it returns false, otherwise true.
    fn push(&mut self, value: T) -> bool;

    /// Removes the element at the provided index and returns it.
    fn remove(&mut self, index: usize) -> Option<T>;

    /// Fill the remaining space of the vector with value.
    fn resize(&mut self, new_len: usize, value: T) -> bool
    where
        T: Clone;

    /// Fill the remaining space of the vector with value.
    fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, f: F) -> bool;

    /// Truncates the vector to `len` and drops all elements right of `len`
    /// in reverse order.
    fn truncate(&mut self, len: usize);
}
