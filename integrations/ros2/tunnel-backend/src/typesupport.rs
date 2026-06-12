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

//! Runtime resolution of rosidl typesupport handles from ROS 2 type names,
//! by loading the per-package typesupport library from the sourced
//! environment.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use libloading::Library;
use r2r_rcl::rosidl_message_type_support_t;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LoadError {
    InvalidTypeName { type_name: String },
    LibraryNotFound { library: String },
    SymbolNotFound { symbol: String },
    NullHandle { type_name: String },
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LoadError::{self:?}")
    }
}

impl core::error::Error for LoadError {}

#[derive(Debug)]
struct Entry {
    handle: *const rosidl_message_type_support_t,
    _library: Library,
}

/// A resolved typesupport handle. The library stays loaded as long as any
/// clone (or the registry) is alive.
#[derive(Debug, Clone)]
pub struct TypeSupport {
    entry: Rc<Entry>,
}

impl TypeSupport {
    pub(crate) fn handle(&self) -> *const rosidl_message_type_support_t {
        self.entry.handle
    }
}

/// Resolves and caches typesupport handles. Entries are never evicted; the
/// registry must outlive all endpoints created from its handles.
#[derive(Debug, Default)]
pub struct TypeSupportRegistry {
    entries: RefCell<HashMap<String, Rc<Entry>>>,
}

impl TypeSupportRegistry {
    pub fn load(&self, type_name: &str) -> Result<TypeSupport, LoadError> {
        if let Some(entry) = self.entries.borrow().get(type_name) {
            return Ok(TypeSupport {
                entry: entry.clone(),
            });
        }

        let entry = Rc::new(load_typesupport_library(type_name)?);
        self.entries
            .borrow_mut()
            .insert(type_name.to_string(), entry.clone());

        Ok(TypeSupport { entry })
    }
}

/// Loads the typesupport library of the type's package and looks up the
/// type's handle in it.
fn load_typesupport_library(type_name: &str) -> Result<Entry, LoadError> {
    let (package, message) = split_type_name(type_name)?;
    let library_name = format!("lib{package}__rosidl_typesupport_c.so");
    let symbol_name =
        format!("rosidl_typesupport_c__get_message_type_support_handle__{package}__msg__{message}");

    // Load the typesupport library, found via the sourced environment's
    // LD_LIBRARY_PATH.
    let library =
        unsafe { Library::new(&library_name) }.map_err(|_| LoadError::LibraryNotFound {
            library: library_name,
        })?;

    // Get the typesupport handle from the loaded library.
    let handle = {
        let get_handle: libloading::Symbol<
            unsafe extern "C" fn() -> *const rosidl_message_type_support_t,
        > = unsafe { library.get(symbol_name.as_bytes()) }.map_err(|_| {
            LoadError::SymbolNotFound {
                symbol: symbol_name,
            }
        })?;
        unsafe { get_handle() }
    };
    if handle.is_null() {
        return Err(LoadError::NullHandle {
            type_name: type_name.to_string(),
        });
    }

    Ok(Entry {
        handle,
        _library: library,
    })
}

fn split_type_name(type_name: &str) -> Result<(&str, &str), LoadError> {
    let mut parts = type_name.split('/');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(package), Some("msg"), Some(message), None)
            if !package.is_empty() && !message.is_empty() =>
        {
            Ok((package, message))
        }
        _ => Err(LoadError::InvalidTypeName {
            type_name: type_name.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::split_type_name;

    #[test]
    fn split_type_name_accepts_message_types() {
        assert_eq!(
            split_type_name("std_msgs/msg/String").unwrap(),
            ("std_msgs", "String")
        );
    }

    #[test]
    fn split_type_name_rejects_other_formats() {
        for invalid in [
            "std_msgs/String",
            "std_msgs/srv/String",
            "/msg/String",
            "std_msgs/msg/",
            "std_msgs/msg/String/extra",
        ] {
            assert!(split_type_name(invalid).is_err(), "{invalid}");
        }
    }
}
