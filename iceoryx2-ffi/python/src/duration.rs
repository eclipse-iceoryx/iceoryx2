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

#[pyclass(str = "{value:?}")]
pub struct Duration {
    pub(crate) value: core::time::Duration,
}

#[pymethods]
impl Duration {
    #[staticmethod]
    pub fn from_micros(micros: u64) -> Duration {
        Self {
            value: core::time::Duration::from_micros(micros),
        }
    }

    #[staticmethod]
    pub fn from_millis(millis: u64) -> Duration {
        Self {
            value: core::time::Duration::from_millis(millis),
        }
    }

    #[staticmethod]
    pub fn from_nanos(nanos: u64) -> Duration {
        Self {
            value: core::time::Duration::from_nanos(nanos),
        }
    }

    #[staticmethod]
    pub fn from_secs(secs: u64) -> Duration {
        Self {
            value: core::time::Duration::from_secs(secs),
        }
    }

    #[staticmethod]
    pub fn from_secs_f64(secs: f64) -> Duration {
        Self {
            value: core::time::Duration::from_secs_f64(secs),
        }
    }

    pub fn as_secs(&self) -> u64 {
        self.value.as_secs()
    }

    pub fn as_secs_f64(&self) -> f64 {
        self.value.as_secs_f64()
    }

    pub fn as_millis(&self) -> u128 {
        self.value.as_millis()
    }

    pub fn as_micros(&self) -> u128 {
        self.value.as_micros()
    }

    pub fn as_nanos(&self) -> u128 {
        self.value.as_nanos()
    }

    pub fn subsec_micros(&self) -> u32 {
        self.value.subsec_micros()
    }

    pub fn subsec_millis(&self) -> u32 {
        self.value.subsec_millis()
    }

    pub fn subsec_nanos(&self) -> u32 {
        self.value.subsec_nanos()
    }
}
