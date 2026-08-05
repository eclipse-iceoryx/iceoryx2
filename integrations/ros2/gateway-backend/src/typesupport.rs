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

use std::collections::HashMap;
use std::rc::Rc;

use iceoryx2_bb_concurrency::cell::RefCell;
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

/// A resolved ROS 2 *typesupport* handle.
///
/// In ROS 2 a "typesupport" is the per-message-type descriptor that rcl and
/// the underlying middleware (DDS) need in order to work with a type. The
/// `rosidl` code generator emits two of them for every `.msg` definition.
/// The regular typesupport carries the function table to (de)serialize the
/// type and the introspection typesupport a description of its fields and
/// the layout of its generated C struct. Both are described by the same C
/// handle type, so `TypeSupport` represents either.
///
/// The gateway must handle whatever message types the user bridges, known only
/// by *name* (a string from config or graph discovery) at runtime, never at
/// compile time. ROS ships each package's typesupports as one compiled shared
/// object each (`lib<pkg>__rosidl_typesupport_c.so`, ...) exporting a
/// C getter function per type, so the only way to obtain the handle for a
/// given name is to `dlopen` that library and resolve the getter symbol at
/// runtime. Without the handle, rcl cannot create a publisher or
/// subscription that DDS peers will match.
///
/// The handle points into the loaded library's memory, so the `Library` is
/// kept alive here. Resolved handles are cached and handed out as shares of
/// one `Rc<TypeSupport>`, keeping each library loaded for as long as any
/// share survives.
#[derive(Debug)]
pub struct TypeSupport {
    handle: *const rosidl_message_type_support_t,
    _library: Library,
}

impl TypeSupport {
    pub(crate) fn handle(&self) -> *const rosidl_message_type_support_t {
        self.handle
    }
}

thread_local! {
    /// Cached typesupport handles resolved so far.
    static TYPESUPPORT: RefCell<HashMap<String, Rc<TypeSupport>>> =
        RefCell::new(HashMap::new());

    /// Cached introspection typesupport handles resolved so far.
    static INTROSPECTION: RefCell<HashMap<String, Rc<TypeSupport>>> =
        RefCell::new(HashMap::new());
}

/// Returns the typesupport handle of `type_name`, driving rcl endpoint
/// creation and `rmw_serialize`/`rmw_deserialize`.
pub(crate) fn load(type_name: &str) -> Result<Rc<TypeSupport>, LoadError> {
    let origin = "typesupport::load";

    if let Some(type_support) = TYPESUPPORT.with(|cache| cache.borrow().get(type_name).cloned()) {
        return Ok(type_support);
    }

    let (package, message) = fail!(
        from origin,
        when split_type_name(type_name),
        "Invalid ROS 2 type name '{}'",
        type_name
    );
    let type_support = Rc::new(load_handle(
        type_name,
        format!("lib{package}__rosidl_typesupport_c.so"),
        format!("rosidl_typesupport_c__get_message_type_support_handle__{package}__msg__{message}"),
    )?);

    TYPESUPPORT.with(|cache| {
        cache
            .borrow_mut()
            .insert(type_name.to_string(), Rc::clone(&type_support));
    });

    Ok(type_support)
}

/// Returns the introspection typesupport handle of `type_name`, describing
/// the members of the type's C struct.
pub(crate) fn load_introspection(type_name: &str) -> Result<Rc<TypeSupport>, LoadError> {
    let origin = "typesupport::load_introspection";

    if let Some(type_support) = INTROSPECTION.with(|cache| cache.borrow().get(type_name).cloned()) {
        return Ok(type_support);
    }

    let (package, message) = fail!(
        from origin,
        when split_type_name(type_name),
        "Invalid ROS 2 type name '{}'",
        type_name
    );
    let type_support = Rc::new(load_handle(
        type_name,
        format!("lib{package}__rosidl_typesupport_introspection_c.so"),
        format!(
            "rosidl_typesupport_introspection_c__get_message_type_support_handle__{package}__msg__{message}"
        ),
    )?);

    INTROSPECTION.with(|cache| {
        cache
            .borrow_mut()
            .insert(type_name.to_string(), Rc::clone(&type_support));
    });

    Ok(type_support)
}

/// Loads `library_name` and retrieves the handle pointer that `symbol_name`
/// returns.
fn load_handle(
    type_name: &str,
    library_name: String,
    symbol_name: String,
) -> Result<TypeSupport, LoadError> {
    let origin = "typesupport::load_handle";

    // Load the typesupport library, found via the sourced environment's
    // LD_LIBRARY_PATH.
    let library = fail!(from origin,
        when unsafe { Library::new(&library_name) },
        with LoadError::LibraryNotFound { library: library_name },
        "Failed to load typesupport library for type '{}'",
        type_name
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
            type_name
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

    Ok(TypeSupport {
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
