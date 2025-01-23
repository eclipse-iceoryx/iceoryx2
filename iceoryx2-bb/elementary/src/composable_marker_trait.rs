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

/// Generates a composable marker trait for the given types. The trait does not only mark the type
/// itself but also slices, arrays, [`Option`], [`Result`], [`core::mem::MaybeUninit`] and
/// [`core::cell::UnsafeCell`] of that type as well as tuples.
///
/// # Example
///
/// ```
/// use iceoryx2_bb_elementary::composable_marker_trait;
///
/// composable_marker_trait! {
///   /// Marker trait for my favorite composable types
///   Favorites => u8, u16, u32, u64
/// }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! composable_marker_trait {
    {
        $(#[$documentation:meta])*
        $name:ident => $($t:ty),*} => {

        $(#[$documentation])*
        pub trait $name {}

        $(impl $name for $t {})*

        impl<T: $name> $name for [T] {}
        impl<T: $name, const N: usize> $name for [T; N] {}
        impl<T: $name> $name for Option<T> {}
        impl<T: $name, E: $name> $name for Result<T, E> {}
        impl<T: $name> $name for core::mem::MaybeUninit<T> {}
        impl<T: $name> $name for core::cell::UnsafeCell<T> {}

        impl<T1: $name, T2: $name> $name for (T1, T2) {}
        impl<T1: $name, T2: $name, T3: $name> $name for (T1, T2, T3) {}
        impl<T1: $name, T2: $name, T3: $name, T4: $name> $name for (T1, T2, T3, T4) {}
        impl<T1: $name, T2: $name, T3: $name, T4: $name, T5: $name> $name for (T1, T2, T3, T4, T5) {}
        impl<T1: $name, T2: $name, T3: $name, T4: $name, T5: $name, T6: $name> $name for (T1, T2, T3, T4, T5, T6) {}
        impl<T1: $name, T2: $name, T3: $name, T4: $name, T5: $name, T6: $name, T7: $name> $name for (T1, T2, T3, T4, T5, T6, T7) {}
    };
}
