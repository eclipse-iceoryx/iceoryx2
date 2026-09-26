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

//! Resolves the directory in which the `.shm_state` files are stored at runtime.
//!
//! Previously the directory was a compile time constant pointing to a system
//! wide temporary directory (e.g. `C:\Temp\` on Windows or `/tmp/` on
//! macOS/FreeBSD) that is shared by all users. This allowed two processes of two
//! different users to collide on the same state files and therefore to break
//! each other's shared memory segments.
//!
//! The directory is now resolved once at runtime in the following order:
//!
//! 1. The `IOX2_SHM_STATE_DIRECTORY` environment variable, if set and non-empty.
//! 2. A per-user default directory:
//!    * Windows: `%APPDATA%\iceoryx2\shm\`
//!    * macOS/FreeBSD: `$XDG_STATE_HOME/iceoryx2/shm/` or, if `XDG_STATE_HOME` is
//!      unset, `$HOME/.local/state/iceoryx2/shm/`
//! 3. The platform's temporary directory (`TEMP_DIRECTORY`) as a fallback.

use iceoryx2_pal_configuration::{PATH_SEPARATOR, TEMP_DIRECTORY};
use std::sync::{Once, OnceLock};

/// Name of the environment variable that can be used to explicitly set the
/// directory in which the `.shm_state` files are stored. This allows to
/// configure the directory at runtime without recompiling or patching the crate.
pub const SHM_STATE_DIRECTORY_ENVIRONMENT_VARIABLE: &str = "IOX2_SHM_STATE_DIRECTORY";

static SHM_STATE_DIRECTORY: OnceLock<Vec<u8>> = OnceLock::new();

/// Returns the runtime resolved directory in which the `.shm_state` files are
/// stored. The returned path always ends with the platform specific path
/// separator.
pub(crate) fn shm_state_directory() -> &'static [u8] {
    SHM_STATE_DIRECTORY
        .get_or_init(resolve_shm_state_directory)
        .as_slice()
}

/// Creates the shared memory state directory if it does not exist yet.
///
/// Panics if the directory can not be created.
pub(crate) fn create_shm_state_directory() {
    static CREATE_ONCE: Once = Once::new();

    CREATE_ONCE.call_once(|| {
        let path = core::str::from_utf8(shm_state_directory()).expect(
            "Non-utf-8 characters are not allowed in the iceoryx2 shared memory state directory",
        );
        let path = std::path::Path::new(path);
        if let Err(error) = std::fs::create_dir_all(path) {
            panic!(
                "Unable to create iceoryx2 shared memory state directory: {path:?}. [{error:?}]"
            );
        }
    });
}

fn resolve_shm_state_directory() -> Vec<u8> {
    resolve_from(
        environment_override_directory(),
        per_user_default_directory(),
    )
}

fn resolve_from(
    override_directory: Option<Vec<u8>>,
    per_user_directory: Option<Vec<u8>>,
) -> Vec<u8> {
    match override_directory.or(per_user_directory) {
        Some(directory) => with_trailing_path_separator(directory),
        None => with_trailing_path_separator(TEMP_DIRECTORY.to_vec()),
    }
}

fn environment_override_directory() -> Option<Vec<u8>> {
    let value = std::env::var_os(SHM_STATE_DIRECTORY_ENVIRONMENT_VARIABLE)?;
    if value.is_empty() {
        return None;
    }

    Some(value.to_string_lossy().into_owned().into_bytes())
}

#[cfg(target_os = "windows")]
fn per_user_default_directory() -> Option<Vec<u8>> {
    let appdata = std::env::var_os("APPDATA")?;
    if appdata.is_empty() {
        return None;
    }

    let mut path = appdata.to_string_lossy().into_owned().into_bytes();
    path.extend_from_slice(br"\iceoryx2\shm");
    Some(path)
}

#[cfg(not(target_os = "windows"))]
fn per_user_default_directory() -> Option<Vec<u8>> {
    let mut path = match std::env::var_os("XDG_STATE_HOME").filter(|value| !value.is_empty()) {
        Some(xdg_state_home) => xdg_state_home.to_string_lossy().into_owned().into_bytes(),
        None => {
            let home = std::env::var_os("HOME").filter(|value| !value.is_empty())?;
            let mut path = home.to_string_lossy().into_owned().into_bytes();
            path.extend_from_slice(b"/.local/state");
            path
        }
    };
    path.extend_from_slice(b"/iceoryx2/shm");
    Some(path)
}

fn with_trailing_path_separator(mut path: Vec<u8>) -> Vec<u8> {
    if path.last() != Some(&PATH_SEPARATOR) {
        path.push(PATH_SEPARATOR);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_trailing_path_separator_appends_missing_separator() {
        let mut expected = b"foo".to_vec();
        expected.push(PATH_SEPARATOR);

        assert_eq!(with_trailing_path_separator(b"foo".to_vec()), expected);
    }

    #[test]
    fn with_trailing_path_separator_does_not_duplicate_separator() {
        let mut path = b"foo".to_vec();
        path.push(PATH_SEPARATOR);

        assert_eq!(with_trailing_path_separator(path.clone()), path);
    }

    #[test]
    fn resolve_from_prefers_override_directory() {
        assert_eq!(
            resolve_from(Some(b"override".to_vec()), Some(b"per_user".to_vec())),
            with_trailing_path_separator(b"override".to_vec())
        );
    }

    #[test]
    fn resolve_from_falls_back_to_per_user_directory() {
        assert_eq!(
            resolve_from(None, Some(b"per_user".to_vec())),
            with_trailing_path_separator(b"per_user".to_vec())
        );
    }

    #[test]
    fn resolve_from_falls_back_to_temp_directory() {
        assert_eq!(
            resolve_from(None, None),
            with_trailing_path_separator(TEMP_DIRECTORY.to_vec())
        );
    }
}
