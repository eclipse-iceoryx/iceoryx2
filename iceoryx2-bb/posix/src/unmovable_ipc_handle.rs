// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_log::fail;
use std::{
    fmt::Debug,
    sync::atomic::{AtomicI64, Ordering},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i64)]
pub enum IpcHandleState {
    Uninitialized = -1,
    PerformingInitialization = 0,
    Initialized = 1,
}

pub(crate) mod internal {
    use super::*;

    pub trait UnmovableIpcHandle: Debug {
        fn is_interprocess_capable(&self) -> bool;
        fn reference_counter(&self) -> &AtomicI64;
    }

    pub trait CreateIpcConstruct<'a, IpcHandle: UnmovableIpcHandle> {
        fn new(handle: &'a IpcHandle) -> Self;
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AcquireIpcHandleError {
    Uninitialized,
    IsNotInterProcessCapable,
}

pub trait IpcCapable<'a, IpcHandle: internal::UnmovableIpcHandle>:
    internal::CreateIpcConstruct<'a, IpcHandle> + Sized + Debug
{
    fn from_ipc_handle(handle: &'a IpcHandle) -> Result<Self, AcquireIpcHandleError> {
        let msg = "Unable to create construct from ipc handle";

        let mut ref_count = handle.reference_counter().load(Ordering::Acquire);
        loop {
            if ref_count <= IpcHandleState::PerformingInitialization as _ {
                fail!(from handle, with AcquireIpcHandleError::Uninitialized,
                "{} since it is not yet initialized.", msg);
            }

            ref_count = match handle.reference_counter().compare_exchange_weak(
                ref_count,
                ref_count + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => v,
            };
        }

        let new_self = Self::new(handle);

        if !handle.is_interprocess_capable() {
            fail!(from handle, with AcquireIpcHandleError::IsNotInterProcessCapable,
                "{} since it is not inter-process capable.", msg);
        }

        Ok(new_self)
    }
}
