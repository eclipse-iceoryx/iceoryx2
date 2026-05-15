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

use alloc::rc::Rc;
use core::ptr::NonNull;
use core::{fmt::Debug, marker::PhantomData, ops::Deref};
use iceoryx2_bb_elementary_traits::non_null::NonNullCompat;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;

use crate::arc_sync_policy::{ArcSyncPolicy, LockGuard};

pub struct Guard<'parent, T: Send + Debug> {
    data: Rc<T>,
    _lifetime: core::marker::PhantomData<&'parent ()>,
}

impl<T: Send + Debug> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

impl<'parent, T: Send + Debug + Abandonable> LockGuard<'parent, T> for Guard<'parent, T> {}

#[derive(Debug)]
pub struct SingleThreaded<T: Send + Debug + Abandonable> {
    data: Rc<T>,
}

impl<T: Send + Debug + Abandonable> Clone for SingleThreaded<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T: Send + Debug + Abandonable> Abandonable for SingleThreaded<T> {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };

        if let Some(value) = Rc::get_mut(&mut this.data) {
            unsafe { T::abandon_in_place(NonNull::iox2_from_mut(value)) };
        } else {
            unsafe { core::ptr::drop_in_place(&mut this.data) };
        }
    }
}

impl<T: Send + Debug + Abandonable> ArcSyncPolicy<T> for SingleThreaded<T> {
    type LockGuard<'parent>
        = Guard<'parent, T>
    where
        Self: 'parent,
        T: 'parent;

    fn new(value: T) -> Result<Self, super::ArcSyncPolicyCreationError> {
        Ok(Self {
            data: Rc::new(value),
        })
    }

    fn lock(&self) -> Self::LockGuard<'_> {
        Guard {
            data: self.data.clone(),
            _lifetime: PhantomData,
        }
    }
}
