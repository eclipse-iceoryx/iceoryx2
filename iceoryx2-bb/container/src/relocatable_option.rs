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

use core::hash::Hash;
use core::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_elementary_traits::{
    placement_default::PlacementDefault, zero_copy_send::ZeroCopySend,
};
use iceoryx2_log::fatal_panic;
use serde::{de::Visitor, Deserialize, Serialize};

/// Implementation of an [`Option`] that is shared-memory compatible,
/// has a stable memory layout and can be used for zero-copy cross-language
/// communication.
///
/// The usage is as close as possible to the original [`Option`], except for
/// the construction via [`Some`] and [`None`].
///
/// # Examples
///
/// ## Construction Comparison
///
/// ```
/// use iceoryx2_bb_container::relocatable_option::RelocatableOption;
///
/// // rust Option
/// fn do_stuff_1(value: i32) -> Option<i32> {
///   if value > 0 {
///     Some(value)
///   } else {
///     None
///   }
/// }
///
/// // RelocatableOption
/// fn do_stuff_2(value: i32) -> RelocatableOption<i32> {
///   if value > 0 {
///     RelocatableOption::Some(value)
///   } else {
///     RelocatableOption::None
///   }
/// }
/// ```
///
/// ## Match Statements
///
/// ```
/// use iceoryx2_bb_container::relocatable_option::RelocatableOption;
///
/// fn do_stuff() -> RelocatableOption<i32> {
///   RelocatableOption::None
/// }
///
/// match do_stuff() {
///   RelocatableOption::Some(v) => println!("{v}"),
///   RelocatableOption::None => println!("none")
/// }
/// ```
#[repr(C, u8)]
#[derive(Default, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum RelocatableOption<T> {
    /// Default value, defines an [`RelocatableOption`] that does contain nothing.
    #[default]
    None,
    /// Defines an [`RelocatableOption`] that contains the provided type `T`.
    Some(T),
}

impl<T> From<Option<T>> for RelocatableOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => RelocatableOption::Some(v),
            None => RelocatableOption::None,
        }
    }
}

impl<T> From<RelocatableOption<T>> for Option<T> {
    fn from(value: RelocatableOption<T>) -> Self {
        value.to_option()
    }
}

impl<T: Serialize> Serialize for RelocatableOption<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Some(v) => serializer.serialize_some(v),
            Self::None => serializer.serialize_none(),
        }
    }
}

struct RelocatableOptionVisitor<T> {
    _data: PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for RelocatableOptionVisitor<T> {
    type Value = RelocatableOption<T>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            formatter,
            "an optional value of type {}",
            core::any::type_name::<T>()
        )
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(RelocatableOption::Some(T::deserialize(deserializer)?))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RelocatableOption::None)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for RelocatableOption<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(RelocatableOptionVisitor { _data: PhantomData })
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for RelocatableOption<T> {}

impl<T: PlacementDefault> PlacementDefault for RelocatableOption<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(RelocatableOption::None)
    }
}

impl<T> RelocatableOption<T> {
    /// Creates a new [`Option`] containing `T`.
    pub fn to_option(self) -> Option<T> {
        match self {
            Self::Some(v) => Some(v),
            Self::None => None,
        }
    }

    /// Returns an [`Option`] with a reference to `T`
    pub fn as_option_ref(&self) -> Option<&T> {
        match self {
            Self::Some(ref v) => Some(v),
            Self::None => None,
        }
    }

    /// Returns an [`Option`] with a mutable reference to `T`
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Some(ref mut v) => Some(v),
            Self::None => None,
        }
    }

    /// Converts the `RelocatableOption<T>` to `RelocatableOption<&T::Target>`.
    pub fn as_deref(&self) -> RelocatableOption<&<T as Deref>::Target>
    where
        T: Deref,
    {
        match self {
            Self::Some(v) => RelocatableOption::Some(v.deref()),
            Self::None => RelocatableOption::None,
        }
    }

    /// Converts the `RelocatableOption<T>` to `RelocatableOption<&mut T::Target>`.
    pub fn as_deref_mut(&mut self) -> RelocatableOption<&mut <T as Deref>::Target>
    where
        T: DerefMut,
    {
        match self {
            Self::Some(v) => RelocatableOption::Some(v.deref_mut()),
            Self::None => RelocatableOption::None,
        }
    }

    /// Returns a [`RelocatableOption`] that contains a mutable reference to `T` if
    /// it holds a value, otherwise it contains nothing.
    pub fn as_mut(&mut self) -> RelocatableOption<&mut T> {
        match self {
            Self::Some(ref mut v) => RelocatableOption::Some(v),
            Self::None => RelocatableOption::None,
        }
    }

    /// Returns a [`RelocatableOption`] that contains a reference to `T` if it holds
    /// a value, otherwise it contains nothing.
    pub fn as_ref(&self) -> RelocatableOption<&T> {
        match self {
            Self::Some(ref v) => RelocatableOption::Some(v),
            Self::None => RelocatableOption::None,
        }
    }

    /// Consumes the [`RelocatableOption`] and returns the contained value `T`. If
    /// it does not contain a value a panic is raised with the provided
    /// message.
    pub fn expect(self, msg: &str) -> T {
        if self.is_none() {
            let origin = alloc::format!(
                "RelocatableOption::<{}>::expect()",
                core::any::type_name::<T>()
            );
            fatal_panic!(from origin, "Expect: {msg}");
        }

        unsafe { self.unwrap_unchecked() }
    }

    /// If the [`RelocatableOption`] contains a value, the provided callback is
    /// called.
    pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Self::Some(data) = &self {
            f(data)
        }

        self
    }

    /// Returns [`true`] if the [`RelocatableOption`] does not contain a value, other
    /// it returns [`false`].
    pub fn is_none(&self) -> bool {
        match self {
            RelocatableOption::None => true,
            RelocatableOption::Some(_v) => false,
        }
    }

    /// Returns [`true`] if the [`RelocatableOption`] does contain a value, other
    /// it returns [`false`].
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Maps a `RelocatableOption<T>` to a `RelocatableOption<U>`
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> RelocatableOption<U> {
        match self {
            RelocatableOption::None => RelocatableOption::None,
            RelocatableOption::Some(v) => RelocatableOption::Some(f(v)),
        }
    }

    /// Replaces the existing value of the [`RelocatableOption`] with the new value.
    /// The old value is returned.
    pub fn replace(&mut self, value: T) -> RelocatableOption<T> {
        let mut new_value = RelocatableOption::Some(value);
        core::mem::swap(self, &mut new_value);
        new_value
    }

    /// Takes the value out of the [`RelocatableOption`] and returns it, leaving an
    /// empty [`RelocatableOption`].
    pub fn take(&mut self) -> RelocatableOption<T> {
        core::mem::take(self)
    }

    /// Takes the value out of the [`RelocatableOption`] if it has a value and the
    /// predicate returns [`true`] leaving an empty [`RelocatableOption`].
    pub fn take_if<P: FnOnce(&mut T) -> bool>(&mut self, predicate: P) -> RelocatableOption<T> {
        match self {
            RelocatableOption::None => RelocatableOption::None,
            RelocatableOption::Some(v) => {
                if predicate(v) {
                    core::mem::take(self)
                } else {
                    RelocatableOption::None
                }
            }
        }
    }

    /// Consumes the [`RelocatableOption`] and returns the value of `T`. If the
    /// [`RelocatableOption`] does not contain a value a panic is raised.
    pub fn unwrap(self) -> T {
        if self.is_none() {
            let origin = alloc::format!(
                "RelocatableOption::<{}>::unwrap()",
                core::any::type_name::<T>()
            );
            fatal_panic!(
                from origin,
                "This should never happen! Accessing the value of an empty RelocatableOption."
            );
        }

        unsafe { self.unwrap_unchecked() }
    }

    /// Consumes the [`RelocatableOption`] and either returns the contained value,
    /// if there is one, otherwise `default` is returned.
    pub fn unwrap_or(self, default: T) -> T {
        if self.is_none() {
            default
        } else {
            unsafe { self.unwrap_unchecked() }
        }
    }

    /// Consumes the [`RelocatableOption`] and either returns the contained value,
    /// if there is one, otherwise `T::default()` is returned.
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        if self.is_none() {
            T::default()
        } else {
            unsafe { self.unwrap_unchecked() }
        }
    }

    /// Consumes the [`RelocatableOption`] and either returns the contained value,
    /// if there is one or returns the return value of the provided callback.
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        if self.is_none() {
            f()
        } else {
            unsafe { self.unwrap_unchecked() }
        }
    }

    /// Consumes the [`RelocatableOption`] and returns the value of `T`.
    ///
    /// # Safety
    ///
    /// *  [`RelocatableOption::is_some()`] == [`true`]
    ///
    pub unsafe fn unwrap_unchecked(self) -> T {
        debug_assert!(self.is_some());
        match self {
            RelocatableOption::Some(v) => v,
            RelocatableOption::None => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}
