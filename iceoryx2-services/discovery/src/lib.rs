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

//! # Discovery Services
//!
//! The `iceoryx2-services-discovery` crate provides discovery services for the components
//! of an iceoryx2 system. These services enable other applications built on iceoryx2 to
//! get informed of the presense of these components.
//!

#![warn(missing_docs)]

/// Discovery and tracking of services in an iceoryx2 system
pub mod service_discovery;
