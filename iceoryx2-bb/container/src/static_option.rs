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
/// ## Construction Comparision
///
/// ```
/// // rust Option
/// fn do_stuff_1(value: i32) -> Option<i32> {
///   if value > 0 {
///     Some(value)
///   } else {
///     None
///   }
/// }
///
/// // StaticOption
/// fn do_stuff_2(value: i32) -> StaticOption<i32> {
///   if value > 0 {
///     StaticOption::Some(value)
///   } else {
///     StaticOption::None
///   }
/// }
/// ```
///
/// ## Match Statements
///
/// The [`StaticOption`] can be converted to the rust [`Option`] with
/// * [`StaticOption::as_option()`]
/// * [`StaticOption::as_option_mut()`]
///
/// to enable the usage in match statements.
///
/// ```
/// fn do_stuff() -> StaticOption<i32> {
///   StaticOption::None
/// }
///
/// match do_stuff().as_option() {
///   Some(v) => println!("{v}"),
///   None => println!("none")
/// }
/// ```
#[repr(C)]
pub struct StaticOption<T> {
    data: MaybeUninit<T>,
    has_contents: u8,
}

impl<T: Hash> Hash for StaticOption<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        if self.is_some() {
            unsafe { self.data.assume_init_ref() }.hash(state)
        } else {
            Option::<T>::None.hash(state)
        }
    }
}

impl<T> Drop for StaticOption<T> {
    fn drop(&mut self) {
        if self.is_some() {
            unsafe { self.data.assume_init_drop() };
        }
    }
}

impl<T: Serialize> Serialize for StaticOption<T> {
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

struct StaticOptionVisitor<T> {
    _data: PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for StaticOptionVisitor<T> {
    type Value = StaticOption<T>;

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
        Ok(StaticOption::some(T::deserialize(deserializer)?))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StaticOption::none())
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for StaticOption<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(StaticOptionVisitor { _data: PhantomData })
    }
}

impl<T> Default for StaticOption<T> {
    fn default() -> Self {
        Self::none()
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for StaticOption<T> {}

impl<T: PlacementDefault> PlacementDefault for StaticOption<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(StaticOption::none())
    }
}

impl<T: PartialEq> PartialEq for StaticOption<T> {
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

impl<T: Eq> Eq for StaticOption<T> {}

impl<T: Clone> Clone for StaticOption<T> {
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

impl<T: Debug> core::fmt::Debug for StaticOption<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_none() {
            write!(f, "StaticOption<{}>::none()", core::any::type_name::<T>())
        } else {
            write!(
                f,
                "StaticOption<{}>::some({:?})",
                core::any::type_name::<T>(),
                self.as_ref().unwrap()
            )
        }
    }
}

impl<T> StaticOption<T> {
    /// Returns an [`Option`] with a reference to `T`
    pub fn as_option(&self) -> Option<&T> {
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

    /// Converts the `StaticOption<T>` to `StaticOption<&T::Target>.
    pub fn as_deref(&self) -> StaticOption<&<T as Deref>::Target>
    where
        T: Deref,
    {
        if self.is_some() {
            StaticOption::some(unsafe { self.data.assume_init_ref().deref() })
        } else {
            StaticOption::none()
        }
    }

    /// Converts the `StaticOption<T>` to `StaticOption<&mut T::Target>.
    pub fn as_deref_mut(&mut self) -> StaticOption<&mut <T as Deref>::Target>
    where
        T: DerefMut,
    {
        if self.is_some() {
            StaticOption::some(unsafe { self.data.assume_init_mut().deref_mut() })
        } else {
            StaticOption::none()
        }
    }

    /// Returns a [`StaticOption`] that contains a mutable reference to `T` if
    /// it holds a value, otherwise it contains nothing.
    pub fn as_mut(&mut self) -> StaticOption<&mut T> {
        if self.is_some() {
            StaticOption::some(unsafe { self.data.assume_init_mut() })
        } else {
            StaticOption::none()
        }
    }

    /// Returns a [`StaticOption`] that contains a reference to `T` if it holds
    /// a value, otherwise it contains nothing.
    pub fn as_ref(&self) -> StaticOption<&T> {
        if self.is_some() {
            StaticOption::some(unsafe { self.data.assume_init_ref() })
        } else {
            StaticOption::none()
        }
    }

    /// Consumes the [`StaticOption`] and returns the contained value `T`. If
    /// it does not contain a value a panic is raised with the provided
    /// message.
    pub fn expect(self, msg: &str) -> T {
        if self.is_none() {
            let origin =
                alloc::format!("StaticOption::<{}>::expect()", core::any::type_name::<T>());
            fatal_panic!(from origin, "Expect: {msg}");
        }

        unsafe { self.unwrap_unchecked() }
    }

    /// Creates a new [`StaticOption`] containing `none`.
    pub fn none() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            has_contents: false as u8,
        }
    }

    /// Creates a new [`StaticOption`] containing the provided value.
    pub fn some(value: T) -> Self {
        Self {
            data: MaybeUninit::new(value),
            has_contents: true as u8,
        }
    }

    /// If the [`StaticOption`] contains a value, the provided callback is
    /// called.
    pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if self.is_some() {
            f(unsafe { self.data.assume_init_ref() })
        }

        self
    }

    /// Returns [`true`] if the [`StaticOption`] does not contain a value, other
    /// it returns [`false`].
    pub fn is_none(&self) -> bool {
        self.has_contents == false as u8
    }

    /// Returns [`true`] if the [`StaticOption`] does contain a value, other
    /// it returns [`false`].
    pub fn is_some(&self) -> bool {
        self.has_contents == true as u8
    }

    /// Maps a `StaticOption<T>` to a `StaticOption<U>`
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> StaticOption<U> {
        if self.is_none() {
            StaticOption::none()
        } else {
            StaticOption::some(f(self.unwrap()))
        }
    }

    /// Replaces the existing value of the [`StaticOption`] with the new value.
    /// The old value is returned.
    pub fn replace(&mut self, value: T) -> StaticOption<T> {
        if self.is_none() {
            self.data.write(value);
            self.has_contents = true as u8;
            StaticOption::none()
        } else {
            StaticOption::some(core::mem::replace(
                unsafe { self.data.assume_init_mut() },
                value,
            ))
        }
    }

    /// Takes the value out of the [`StaticOption`] and returns it, leaving an
    /// empty [`StaticOption`].
    pub fn take(&mut self) -> StaticOption<T> {
        if self.is_none() {
            StaticOption::none()
        } else {
            self.has_contents = false as u8;
            StaticOption {
                data: core::mem::replace(&mut self.data, MaybeUninit::uninit()),
                has_contents: true as u8,
            }
        }
    }

    /// Takes the value out of the [`StaticOption`] if it has a value and the
    /// predicate returns [`true`] leaving an empty [`StaticOption`].
    pub fn take_if<P: FnOnce(&mut T) -> bool>(&mut self, predicate: P) -> StaticOption<T> {
        if self.is_none() {
            StaticOption::none()
        } else if predicate(unsafe { self.data.assume_init_mut() }) {
            self.has_contents = false as u8;
            StaticOption {
                data: core::mem::replace(&mut self.data, MaybeUninit::uninit()),
                has_contents: true as u8,
            }
        } else {
            StaticOption::none()
        }
    }

    /// Consumes the [`StaticOption`] and returns the value of `T`. If the
    /// [`StaticOption`] does not contain a value a panic is raised.
    pub fn unwrap(self) -> T {
        if self.is_none() {
            let origin =
                alloc::format!("StaticOption::<{}>::unwrap()", core::any::type_name::<T>());
            fatal_panic!(
                from origin,
                "This should never happen! Accessing the value of an empty StaticOption."
            );
        }

        unsafe { self.unwrap_unchecked() }
    }

    /// Consumes the [`StaticOption`] and either returns the contained value,
    /// if there is one, otherwise `default` is returned.
    pub fn unwrap_or(self, default: T) -> T {
        if self.is_none() {
            default
        } else {
            unsafe { self.unwrap_unchecked() }
        }
    }

    /// Consumes the [`StaticOption`] and either returns the contained value,
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

    /// Consumes the [`StaticOption`] and either returns the contained value,
    /// if there is one or returns the return value of the provided callback.
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        if self.is_none() {
            f()
        } else {
            unsafe { self.unwrap_unchecked() }
        }
    }

    /// Consumes the [`StaticOption`] and returns the value of `T`.
    ///
    /// # Safety
    ///
    /// *  [`StaticOption::is_some()`] == [`true`]
    ///
    pub unsafe fn unwrap_unchecked(mut self) -> T {
        debug_assert!(self.is_some());
        self.has_contents = false as u8;
        self.data.assume_init_read()
    }
}
