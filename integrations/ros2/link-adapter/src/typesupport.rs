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
use std::collections::HashMap;
use std::rc::Rc;
use std::thread::LocalKey;

use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::{fail, origin};

use libloading::Library;
use r2r_rcl::{
    rosidl_message_type_support_t, rosidl_service_type_support_t,
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
};

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

/// The kinds of typesupport that can be loaded.
pub trait TypeSupportKind: private::Sealed + Sized + 'static {
    type Handle;
    /// The typesupport library of a package, `lib<package>__<LIBRARY>.so`.
    const LIBRARY: &'static str;
    /// The getter of a handle in it, `<LIBRARY>__<GETTER>__<package>__<kind>__<Type>`.
    const GETTER: &'static str;

    /// The typesupports of this kind opened so far, one map per thread.
    fn opened() -> &'static LocalKey<RefCell<OpenedTypeSupports<Self>>>;
}

/// A resolved ROS 2 *typesupport* handle.
///
/// In ROS 2 a "typesupport" is the per-type descriptor that rcl and the
/// underlying middleware (DDS) need in order to work with a type. There are
/// three kinds of typesupports that can be generated with `rosidl`, which
/// are distinguished here via [`TypeSupportKind`].
///
/// A message's typesupport provides the mechanisms for a type's
/// (de)serialization. A service's typesupport does the same for its request
/// and response types. The introspection typesupport provides mechanisms to
/// describe the C struct of the type.
///
/// Types are identified by name. The typesupport library of a type is
/// loaded by its name at runtime. Opened handles are cached and shared
/// between endpoints of the same type. Once loaded, the library stays open for
/// as long as the gateway runs.
#[derive(Debug)]
pub struct TypeSupport<K: TypeSupportKind> {
    /// The rosidl handle to the loaded typesupport.
    handle: *const K::Handle,
    /// The loaded library the handle points into.
    _library: Library,
}

impl<K: TypeSupportKind> TypeSupport<K> {
    pub(crate) fn handle(&self) -> *const K::Handle {
        self.handle
    }
}

/// The typesupport of a ROS 2 message.
pub type MessageTypeSupport = TypeSupport<MessageKind>;

/// The introspection typesupport of a ROS 2 message.
pub type IntrospectionTypeSupport = TypeSupport<IntrospectionKind>;

/// The typesupport of a ROS 2 service, covering its request and its
/// response.
pub type ServiceTypeSupport = TypeSupport<ServiceKind>;

impl IntrospectionTypeSupport {
    /// The members of the type's C struct.
    pub(crate) fn members(&self) -> *const MessageMembers {
        // SAFETY: the handle points into the loaded library, and the data of
        // an introspection typesupport is the members struct rosidl
        // generates.
        unsafe { (*self.handle).data }.cast::<MessageMembers>()
    }
}

/// The typesupports of one kind opened so far, by type name, failed opens
/// kept and not retried.
pub type OpenedTypeSupports<K> = HashMap<String, Result<Rc<TypeSupport<K>>, LoadError>>;

/// The typesupport of a message.
#[derive(Debug)]
pub struct MessageKind;

impl TypeSupportKind for MessageKind {
    type Handle = rosidl_message_type_support_t;
    const LIBRARY: &'static str = "rosidl_typesupport_c";
    const GETTER: &'static str = "get_message_type_support_handle";

    fn opened() -> &'static LocalKey<RefCell<OpenedTypeSupports<Self>>> {
        thread_local! {
            static OPENED: RefCell<OpenedTypeSupports<MessageKind>> = RefCell::new(HashMap::new());
        }
        &OPENED
    }
}

/// The introspection typesupport of a message.
#[derive(Debug)]
pub struct IntrospectionKind;

impl TypeSupportKind for IntrospectionKind {
    type Handle = rosidl_message_type_support_t;
    const LIBRARY: &'static str = "rosidl_typesupport_introspection_c";
    const GETTER: &'static str = "get_message_type_support_handle";

    fn opened() -> &'static LocalKey<RefCell<OpenedTypeSupports<Self>>> {
        thread_local! {
            static OPENED: RefCell<OpenedTypeSupports<IntrospectionKind>> = RefCell::new(HashMap::new());
        }
        &OPENED
    }
}

/// The typesupport of a service.
#[derive(Debug)]
pub struct ServiceKind;

impl TypeSupportKind for ServiceKind {
    type Handle = rosidl_service_type_support_t;
    const LIBRARY: &'static str = "rosidl_typesupport_c";
    const GETTER: &'static str = "get_service_type_support_handle";

    fn opened() -> &'static LocalKey<RefCell<OpenedTypeSupports<Self>>> {
        thread_local! {
            static OPENED: RefCell<OpenedTypeSupports<ServiceKind>> = RefCell::new(HashMap::new());
        }
        &OPENED
    }
}

/// The typesupport of kind `K` of `type_name`, opened on first use.
///
/// A failed open is kept and not retried.
pub(crate) fn get<K: TypeSupportKind>(type_name: &str) -> Result<Rc<TypeSupport<K>>, LoadError> {
    if let Some(result) = K::opened().with(|opened| opened.borrow().get(type_name).cloned()) {
        return result;
    }

    let result = open::<K>(type_name);

    K::opened().with(|opened| {
        opened
            .borrow_mut()
            .insert(type_name.to_string(), result.clone());
    });

    result
}

/// Opens the typesupport library of `type_name` and calls the getter of
/// kind `K` in it for the handle.
///
/// The library is kept with the handle it keeps valid.
fn open<K: TypeSupportKind>(type_name: &str) -> Result<Rc<TypeSupport<K>>, LoadError> {
    let origin = origin!("open");

    let (package, kind, name) = fail!(
        from origin,
        when split_type_name(type_name),
        "Invalid ROS 2 type name '{}'",
        type_name
    );
    let library_name = format!("lib{package}__{}.so", K::LIBRARY);
    let symbol_name = format!("{}__{}__{package}__{kind}__{name}", K::LIBRARY, K::GETTER);

    // The library is found via the sourced environment's LD_LIBRARY_PATH.
    let library = match unsafe { Library::new(&library_name) } {
        Ok(library) => library,
        Err(error) => {
            fail!(
                from origin,
                with LoadError::LibraryNotFound { library: library_name },
                "Failed to load typesupport library for type '{}': {}",
                type_name,
                error
            );
        }
    };

    let handle = {
        let get_handle: libloading::Symbol<unsafe extern "C" fn() -> *const K::Handle> =
            match unsafe { library.get(symbol_name.as_bytes()) } {
                Ok(symbol) => symbol,
                Err(error) => {
                    fail!(
                        from origin,
                        with LoadError::SymbolNotFound { symbol: symbol_name },
                        "Failed to resolve typesupport symbol for type '{}': {}",
                        type_name,
                        error
                    );
                }
            };
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

    Ok(Rc::new(TypeSupport {
        handle,
        _library: library,
    }))
}

/// The package, the kind (`msg` or `srv`) and the type of a type name.
fn split_type_name(type_name: &str) -> Result<(&str, &str, &str), LoadError> {
    let origin = origin!("split_type_name");

    let mut parts = type_name.split('/');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(package), Some(kind @ ("msg" | "srv")), Some(name), None)
            if !package.is_empty() && !name.is_empty() =>
        {
            Ok((package, kind, name))
        }
        _ => {
            fail!(
                from origin,
                with LoadError::InvalidTypeName { type_name: type_name.to_string() },
                "Type name '{}' is not of the form package/msg/Message or package/srv/Service", type_name
            );
        }
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::MessageKind {}
    impl Sealed for super::IntrospectionKind {}
    impl Sealed for super::ServiceKind {}
}

#[cfg(test)]
mod tests {
    use super::split_type_name;

    #[test]
    fn split_type_name_accepts_message_and_service_types() {
        assert_eq!(
            split_type_name("std_msgs/msg/String").unwrap(),
            ("std_msgs", "msg", "String")
        );
        assert_eq!(
            split_type_name("example_interfaces/srv/AddTwoInts_Request").unwrap(),
            ("example_interfaces", "srv", "AddTwoInts_Request")
        );
    }

    #[test]
    fn split_type_name_rejects_other_formats() {
        for invalid in [
            "std_msgs/String",
            "std_msgs/action/Thing",
            "/msg/String",
            "std_msgs/msg/",
            "std_msgs/msg/String/extra",
        ] {
            assert!(split_type_name(invalid).is_err(), "{invalid}");
        }
    }
}
