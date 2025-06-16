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

use pyo3::prelude::*;

#[pyclass(str = "{0:?}", eq)]
#[derive(PartialEq)]
/// Represents a time duration.
pub struct Duration(pub(crate) core::time::Duration);

#[pymethods]
impl Duration {
    #[staticmethod]
    /// Creates a new `Duration` from a given number of micro seconds
    pub fn from_micros(micros: u64) -> Duration {
        Self(core::time::Duration::from_micros(micros))
    }

    #[staticmethod]
    /// Creates a new `Duration` from a given number of milli seconds
    pub fn from_millis(millis: u64) -> Duration {
        Self(core::time::Duration::from_millis(millis))
    }

    #[staticmethod]
    /// Creates a new `Duration` from a given number of nano seconds
    pub fn from_nanos(nanos: u64) -> Duration {
        Self(core::time::Duration::from_nanos(nanos))
    }

    #[staticmethod]
    /// Creates a new `Duration` from a given number of seconds
    pub fn from_secs(secs: u64) -> Duration {
        Self(core::time::Duration::from_secs(secs))
    }

    #[staticmethod]
    /// Creates a new `Duration` from a given number of seconds
    pub fn from_secs_f64(secs: f64) -> Duration {
        Self(core::time::Duration::from_secs_f64(secs))
    }

    /// Returns the number of seconds stored in the `Duration`
    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }

    /// Returns the number of seconds stored in the `Duration`
    pub fn as_secs_f64(&self) -> f64 {
        self.0.as_secs_f64()
    }

    /// Returns the number of milli seconds stored in the `Duration`
    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }

    /// Returns the number of micro seconds stored in the `Duration`
    pub fn as_micros(&self) -> u128 {
        self.0.as_micros()
    }

    /// Returns the number of nano seconds stored in the `Duration`
    pub fn as_nanos(&self) -> u128 {
        self.0.as_nanos()
    }

    /// Returns the fractional micro seconds part stored in the `Duration`
    pub fn subsec_micros(&self) -> u32 {
        self.0.subsec_micros()
    }

    /// Returns the fractional milli seconds part stored in the `Duration`
    pub fn subsec_millis(&self) -> u32 {
        self.0.subsec_millis()
    }

    /// Returns the fractional nano seconds part stored in the `Duration`
    pub fn subsec_nanos(&self) -> u32 {
        self.0.subsec_nanos()
    }
}
