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

mod layout;
mod read;
mod write;

pub use layout::{Layout, MemberEntry, MemberKind};
pub use read::{Locations, MessageElementIter, ReadError, StringElementIter, locate};
pub use write::{WriteError, write};
