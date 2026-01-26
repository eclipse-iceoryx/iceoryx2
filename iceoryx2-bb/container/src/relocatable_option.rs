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
use core::mem::MaybeUninit;
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
/// use iceoryx2_bb_container::static_option::RelocatableOption;
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
///     RelocatableOption::some(value)
///   } else {
///     RelocatableOption::none()
///   }
/// }
/// ```
///
/// ## Match Statements
///
/// The [`RelocatableOption`] can be converted to the rust [`Option`] with
/// * [`RelocatableOption::as_option_ref()`]
/// * [`RelocatableOption::as_option_mut()`]
///
/// to enable the usage in match statements.
///
/// ```
/// use iceoryx2_bb_container::static_option::RelocatableOption;
///
/// fn do_stuff() -> RelocatableOption<i32> {
///   RelocatableOption::none()
/// }
///
/// match do_stuff().as_option_ref() {
///   Some(v) => println!("{v}"),
///   None => println!("none")
/// }
/// ```
#[repr(C)]
pub struct RelocatableOption<T> {
    data: MaybeUninit<T>,
    has_contents: u8,
}

impl<T> From<Option<T>> for RelocatableOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => RelocatableOption::some(v),
            None => RelocatableOption::none(),
        }
    }
}

impl<T> From<RelocatableOption<T>> for Option<T> {
    fn from(value: RelocatableOption<T>) -> Self {
        value.to_option()
    }
}

impl<T: Hash> Hash for RelocatableOption<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        if self.is_some() {
            unsafe { self.data.assume_init_ref() }.hash(state)
        } else {
            Option::<T>::None.hash(state)
        }
    }
}

impl<T> Drop for RelocatableOption<T> {
    fn drop(&mut self) {
        if self.is_some() {
            unsafe { core::ptr::drop_in_place(self.data.as_mut_ptr()) };
        }
    }
}

impl<T: Serialize> Serialize for RelocatableOption<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_some() {
            serializer.serialize_some(unsafe { self.data.assume_init_ref() })
        } else {
            serializer.serialize_none()
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
        Ok(RelocatableOption::some(T::deserialize(deserializer)?))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RelocatableOption::none())
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

impl<T> Default for RelocatableOption<T> {
    fn default() -> Self {
        Self::none()
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for RelocatableOption<T> {}

impl<T: PlacementDefault> PlacementDefault for RelocatableOption<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(RelocatableOption::none())
    }
}

impl<T: PartialEq> PartialEq for RelocatableOption<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.has_contents != other.has_contents {
            return false;
        }

        if self.has_contents == false as u8 {
            return true;
        }

        self.as_ref().unwrap() == other.as_ref().unwrap()
    }
}

impl<T: Eq> Eq for RelocatableOption<T> {}

impl<T: Clone> Clone for RelocatableOption<T> {
    fn clone(&self) -> Self {
        Self {
            data: if self.is_some() {
                MaybeUninit::new(unsafe { self.data.assume_init_ref() }.clone())
            } else {
                MaybeUninit::uninit()
            },
            has_contents: self.has_contents,
        }
    }
}

impl<T: Debug> core::fmt::Debug for RelocatableOption<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_none() {
            write!(
                f,
                "RelocatableOption<{}>::none()",
                core::any::type_name::<T>()
            )
        } else {
            write!(
                f,
                "RelocatableOption<{}>::some({:?})",
                core::any::type_name::<T>(),
                self.as_ref().unwrap()
            )
        }
    }
}

impl<T> RelocatableOption<T> {
    /// Creates a new [`Option`] containing `T`.
    pub fn to_option(self) -> Option<T> {
        if self.is_none() {
            None
        } else {
            Some(self.unwrap())
        }
    }

    /// Returns an [`Option`] with a reference to `T`
    pub fn as_option_ref(&self) -> Option<&T> {
        if self.is_some() {
            Some(unsafe { self.data.assume_init_ref() })
        } else {
            None
        }
    }

    /// Returns an [`Option`] with a mutable reference to `T`
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        if self.is_some() {
            Some(unsafe { self.data.assume_init_mut() })
        } else {
            None
        }
    }

    /// Converts the `RelocatableOption<T>` to `RelocatableOption<&T::Target>`.
    pub fn as_deref(&self) -> RelocatableOption<&<T as Deref>::Target>
    where
        T: Deref,
    {
        if self.is_some() {
            RelocatableOption::some(unsafe { self.data.assume_init_ref().deref() })
        } else {
            RelocatableOption::none()
        }
    }

    /// Converts the `RelocatableOption<T>` to `RelocatableOption<&mut T::Target>`.
    pub fn as_deref_mut(&mut self) -> RelocatableOption<&mut <T as Deref>::Target>
    where
        T: DerefMut,
    {
        if self.is_some() {
            RelocatableOption::some(unsafe { self.data.assume_init_mut().deref_mut() })
        } else {
            RelocatableOption::none()
        }
    }

    /// Returns a [`RelocatableOption`] that contains a mutable reference to `T` if
    /// it holds a value, otherwise it contains nothing.
    pub fn as_mut(&mut self) -> RelocatableOption<&mut T> {
        if self.is_some() {
            RelocatableOption::some(unsafe { self.data.assume_init_mut() })
        } else {
            RelocatableOption::none()
        }
    }

    /// Returns a [`RelocatableOption`] that contains a reference to `T` if it holds
    /// a value, otherwise it contains nothing.
    pub fn as_ref(&self) -> RelocatableOption<&T> {
        if self.is_some() {
            RelocatableOption::some(unsafe { self.data.assume_init_ref() })
        } else {
            RelocatableOption::none()
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

    /// Creates a new [`RelocatableOption`] containing `none`.
    pub fn none() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            has_contents: false as u8,
        }
    }

    /// Creates a new [`RelocatableOption`] containing the provided value.
    pub fn some(value: T) -> Self {
        Self {
            data: MaybeUninit::new(value),
            has_contents: true as u8,
        }
    }

    /// If the [`RelocatableOption`] contains a value, the provided callback is
    /// called.
    pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if self.is_some() {
            f(unsafe { self.data.assume_init_ref() })
        }

        self
    }

    /// Returns [`true`] if the [`RelocatableOption`] does not contain a value, other
    /// it returns [`false`].
    pub fn is_none(&self) -> bool {
        self.has_contents == false as u8
    }

    /// Returns [`true`] if the [`RelocatableOption`] does contain a value, other
    /// it returns [`false`].
    pub fn is_some(&self) -> bool {
        self.has_contents == true as u8
    }

    /// Maps a `RelocatableOption<T>` to a `RelocatableOption<U>`
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> RelocatableOption<U> {
        if self.is_none() {
            RelocatableOption::none()
        } else {
            RelocatableOption::some(f(self.unwrap()))
        }
    }

    /// Replaces the existing value of the [`RelocatableOption`] with the new value.
    /// The old value is returned.
    pub fn replace(&mut self, value: T) -> RelocatableOption<T> {
        if self.is_none() {
            self.data.write(value);
            self.has_contents = true as u8;
            RelocatableOption::none()
        } else {
            RelocatableOption::some(core::mem::replace(
                unsafe { self.data.assume_init_mut() },
                value,
            ))
        }
    }

    /// Takes the value out of the [`RelocatableOption`] and returns it, leaving an
    /// empty [`RelocatableOption`].
    pub fn take(&mut self) -> RelocatableOption<T> {
        if self.is_none() {
            RelocatableOption::none()
        } else {
            self.has_contents = false as u8;
            RelocatableOption {
                data: core::mem::replace(&mut self.data, MaybeUninit::uninit()),
                has_contents: true as u8,
            }
        }
    }

    /// Takes the value out of the [`RelocatableOption`] if it has a value and the
    /// predicate returns [`true`] leaving an empty [`RelocatableOption`].
    pub fn take_if<P: FnOnce(&mut T) -> bool>(&mut self, predicate: P) -> RelocatableOption<T> {
        if self.is_none() {
            RelocatableOption::none()
        } else if predicate(unsafe { self.data.assume_init_mut() }) {
            self.has_contents = false as u8;
            RelocatableOption {
                data: core::mem::replace(&mut self.data, MaybeUninit::uninit()),
                has_contents: true as u8,
            }
        } else {
            RelocatableOption::none()
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
    pub unsafe fn unwrap_unchecked(mut self) -> T {
        debug_assert!(self.is_some());
        self.has_contents = false as u8;
        self.data.assume_init_read()
    }
}
