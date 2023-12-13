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

use windows_sys::Win32::Foundation::MAX_PATH;

pub(crate) const MAX_PATH_LENGTH: usize = MAX_PATH as usize;
pub(crate) const SHM_STATE_DIRECTORY: &[u8] = iceoryx2_pal_configuration::TEMP_DIRECTORY;
pub(crate) const SHM_STATE_SUFFIX: &[u8] = b".shm_state";
#[doc(hidden)]
pub const FD_SET_CAPACITY: usize = 64;
