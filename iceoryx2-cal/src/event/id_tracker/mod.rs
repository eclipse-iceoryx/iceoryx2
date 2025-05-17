// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

pub mod bit_set;

use core::fmt::Debug;

use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;

use super::{NotifierNotifyError, TriggerId};

/// The [`IdTracker`] is a building block for [`crate::event::Event`]
/// concept. Its task is to track the origin of the signal that was sent
/// via the [`crate::event::signal_mechanism::SignalMechanism`].
pub trait IdTracker: RelocatableContainer + Send + Sync + Debug {
    /// Returns the max value of a [`TriggerId`] that can be tracked with the
    /// [`IdTracker`].
    fn trigger_id_max(&self) -> TriggerId;

    /// Tracks the provided [`TriggerId`].
    ///
    /// # Safety
    ///  * underlying container must be initialized with [`RelocatableContainer::init()`]
    ///
    unsafe fn add(&self, id: TriggerId) -> Result<(), NotifierNotifyError>;

    /// Acquires a tracked [`TriggerId`]. When no [`TriggerId`]s are tracked
    /// or all tracked [`TriggerId`]s have been acquired it returns [`None`].
    ///
    /// # Safety
    ///  * underlying container must be initialized with [`RelocatableContainer::init()`]
    ///
    unsafe fn acquire(&self) -> Option<TriggerId>;

    /// Acquires all tracked [`TriggerId`]s and calls for everyone the user
    /// provided callback with the [`TriggerId`] as input argument.
    ///
    /// # Safety
    ///  * underlying container must be initialized with [`RelocatableContainer::init()`]
    ///
    unsafe fn acquire_all<F: FnMut(TriggerId)>(&self, callback: F);
}
