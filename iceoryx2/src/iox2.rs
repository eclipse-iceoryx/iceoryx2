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

//! # Example
//!
//! ## Simple Event Loop
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
//!     // your algorithm in here
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Event Loop
//!
//! ```no_run
//! use core::time::Duration;
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const CYCLE_TIME: Duration = Duration::from_secs(1);
//!
//! loop {
//!     match Iox2::wait(CYCLE_TIME) {
//!         Iox2Event::Tick => {
//!             println!("entered next cycle");
//!         }
//!         Iox2Event::TerminationRequest => {
//!             println!("User pressed CTRL+c, terminating");
//!             break;
//!         }
//!         Iox2Event::InterruptSignal => {
//!             println!("Someone send an interrupt signal ...");
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use core::time::Duration;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_posix::clock::{nanosleep, NanosleepError};
use iceoryx2_bb_posix::signal::SignalHandler;

/// A complete list of all events that can occur in the main event loop, [`Iox2::wait()`].
pub enum Iox2Event {
    Tick,
    TerminationRequest,
    InterruptSignal,
}

/// The main event loop handling mechanism.
#[derive(Debug)]
#[non_exhaustive]
pub struct Iox2 {}

impl Iox2 {
    fn get_instance() -> &'static Self {
        static INSTANCE: Iox2 = Iox2 {};
        &INSTANCE
    }

    fn wait_impl(&self, cycle_time: Duration) -> Iox2Event {
        if SignalHandler::termination_requested() {
            return Iox2Event::TerminationRequest;
        }

        match nanosleep(cycle_time) {
            Ok(()) => {
                if SignalHandler::termination_requested() {
                    Iox2Event::TerminationRequest
                } else {
                    Iox2Event::Tick
                }
            }
            Err(NanosleepError::InterruptedBySignal(_)) => Iox2Event::InterruptSignal,
            Err(v) => {
                fatal_panic!(from self,
                    "Failed to wait with cycle time {:?} in main event look, caused by ({:?}).",
                    cycle_time, v);
            }
        }
    }

    /// Waits until an event has received. It returns
    /// [`Iox2Event::Tick`] when the `cycle_time` has passed, otherwise the other event that
    /// can occur.
    pub fn wait(cycle_time: Duration) -> Iox2Event {
        Self::get_instance().wait_impl(cycle_time)
    }
}
