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

use iceoryx2_log::fail;
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

/// A resolved ROS 2 *typesupport* handle.
///
/// In ROS 2 a "typesupport" is the per-message-type descriptor that rcl and
/// the underlying middleware (DDS) need in order to work with a type: its
/// fully-qualified name, a type hash used to match endpoints, and the function
/// table to (de)serialize it. The `rosidl` code generator emits one for every
/// `.msg` definition.
///
/// The tunnel must handle whatever message types the user bridges, known only
/// by *name* (a string from config or graph discovery) at runtime, never at
/// compile time. ROS ships each package's typesupport as a compiled shared
/// object (`lib<pkg>__rosidl_typesupport_c.so`) exporting a C getter function
/// per type, so the only way to obtain the handle for a given name is to
/// `dlopen` that library and resolve the getter symbol at runtime. Without
/// the handle, rcl cannot create a publisher or subscription that DDS peers
/// will match.
///
/// The handle points into the loaded library's memory, so the `Library` is
/// kept alive here; it stays loaded as long as any clone is alive.
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
    let origin = "TypeSupportRegistry::load";

    let (package, message) = fail!(
        from origin,
        when split_type_name(type_name),
        "Invalid ROS 2 type name '{}'",
        type_name
    );
    let library_name = format!("lib{package}__rosidl_typesupport_c.so");
    let symbol_name =
        format!("rosidl_typesupport_c__get_message_type_support_handle__{package}__msg__{message}");

    // Load the typesupport library, found via the sourced environment's
    // LD_LIBRARY_PATH.
    let library = fail!(from origin,
        when unsafe { Library::new(&library_name) },
        with LoadError::LibraryNotFound { library: library_name },
        "Failed to load typesupport library for package '{}'",
        package
    );

    // Get the typesupport handle from the loaded library.
    let handle = {
        let get_handle: libloading::Symbol<
            unsafe extern "C" fn() -> *const rosidl_message_type_support_t,
        > = fail!(
            from origin,
            when unsafe { library.get(symbol_name.as_bytes()) },
            with LoadError::SymbolNotFound { symbol: symbol_name },
            "Failed to resolve typesupport symbol for type '{}'",
            message
        );
        unsafe { get_handle() }
    };
    if handle.is_null() {
        fail!(
            from origin,
            with LoadError::NullHandle { type_name: type_name.to_string() },
            "Typesupport handle for '{}' is null",
            type_name
        );
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
