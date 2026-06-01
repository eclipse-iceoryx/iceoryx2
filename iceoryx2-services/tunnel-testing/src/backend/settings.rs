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

#[cfg(all(not(target_os = "windows"), not(target_os = "nto")))]
pub const ROOT_DIR: &[u8] = b"/tmp/iceoryx2/tunnels/test-backend";
#[cfg(target_os = "nto")]
pub const ROOT_DIR: &[u8] = b"/data/iceoryx2/tunnels/test-backend";
#[cfg(target_os = "windows")]
pub const ROOT_DIR: &[u8] = b"C:\\Temp\\iceoryx2\\tunnels\\test-backend";

pub const SESSIONS_DIR_NAME: &[u8] = b"sessions";
pub const SERVICES_DIR_NAME: &[u8] = b"services";
pub const LOCKFILE_NAME: &[u8] = b"session.lock";
pub const SOCKET_NAME: &[u8] = b"session.sock";
pub const MAX_DATAGRAM: usize = 256 * 1024;
