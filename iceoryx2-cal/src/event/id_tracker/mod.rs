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

use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;

use super::{NotifierNotifyError, TriggerId};

pub trait IdTracker: RelocatableContainer + Send + Sync {
    fn trigger_id_max(&self) -> TriggerId;
    fn add(&self, id: TriggerId) -> Result<(), NotifierNotifyError>;
    fn acquire(&self) -> Option<TriggerId>;
    fn acquire_all<F: FnMut(TriggerId)>(&self, callback: F);
}
