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

use core::fmt;

#[repr(C)]
pub(crate) struct Message<Header, Data> {
    pub(crate) header: Header,
    pub(crate) data: Data,
}

impl<Header: fmt::Debug, Data: fmt::Debug> fmt::Debug for Message<Header, Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message<Header, Data>")
            .field("header", &self.header)
            .field("data", &self.data)
            .finish()
    }
}
