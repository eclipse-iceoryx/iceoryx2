// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#[generic_tests::define]
mod service_builder {
    use crate::api::*;
    use crate::tests::{ServiceTypeMapping, create_event_service, create_node};
    use core::ffi::c_void;
    use iceoryx2::prelude::*;
    use iceoryx2_bb_testing::assert_that;

    struct VisitContext {
        target_name: &'static str,
        key_storage: *mut iox2_listener_key_t,
        key: iox2_listener_key_h,
        visits: usize,
        notify_result: i32,
        stop: bool,
    }

    extern "C" fn visit_listener(
        context: iox2_callback_context,
        monofier: iox2_monofier_ptr,
        details: iox2_listener_details_ptr,
    ) -> iox2_callback_progression_e {
        unsafe {
            let context = &mut *(context as *mut VisitContext);
            context.visits += 1;
            let name = iox2_listener_details_listener_name(details);
            let mut len = 0;
            let chars = iox2_port_name_as_chars(name, &mut len);
            let name = core::str::from_utf8(core::slice::from_raw_parts(chars, len)).unwrap();
            if name == context.target_name {
                iox2_monofier_listener_key(monofier, context.key_storage, &mut context.key);
                context.notify_result = iox2_monofier_notify_with_custom_event_id(
                    monofier,
                    &iox2_event_id_t { value: 3 },
                );
                if context.stop {
                    return iox2_callback_progression_e::STOP;
                }
            }
            iox2_callback_progression_e::CONTINUE
        }
    }

    extern "C" fn count_event(
        event: *const iox2_event_id_t,
        count: u64,
        context: iox2_callback_context,
    ) {
        unsafe {
            let values = &mut *(context as *mut (u64, usize));
            values.0 += count;
            values.1 = (*event).value;
        }
    }

    unsafe fn listener<S: Service + ServiceTypeMapping>(
        factory: iox2_port_factory_event_h_ref,
        name: &str,
    ) -> iox2_listener_h {
        unsafe {
            let builder = iox2_port_factory_event_listener_builder(factory, core::ptr::null_mut());
            let mut name_handle = core::ptr::null_mut();
            assert_eq!(
                iox2_port_name_new(
                    core::ptr::null_mut(),
                    name.as_ptr() as _,
                    name.len(),
                    &mut name_handle
                ),
                IOX2_OK
            );
            iox2_port_factory_listener_builder_set_name(
                &builder,
                iox2_cast_port_name_ptr(name_handle),
            );
            iox2_port_name_drop(name_handle);
            let mut handle = core::ptr::null_mut();
            assert_eq!(
                iox2_port_factory_listener_builder_create(
                    builder,
                    core::ptr::null_mut(),
                    &mut handle
                ),
                IOX2_OK
            );
            handle
        }
    }

    unsafe fn events(handle: iox2_listener_h_ref) -> (u64, usize) {
        unsafe {
            let mut observed = (0, 0);
            let mut notifications = 0;
            assert_eq!(
                iox2_listener_try_wait(
                    handle,
                    &mut notifications,
                    count_event,
                    &mut observed as *mut _ as *mut c_void
                ),
                IOX2_OK
            );
            observed
        }
    }

    #[test]
    fn basic_notifier_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("bar");

            let event_service_handle = create_event_service(&node_handle, "all/glory/to/hypnotaod");

            let notifier_builder_handle = iox2_port_factory_event_notifier_builder(
                &event_service_handle,
                core::ptr::null_mut(),
            );

            let mut notifier_handle = core::ptr::null_mut();
            let ret_val = iox2_port_factory_notifier_builder_create(
                notifier_builder_handle,
                core::ptr::null_mut(),
                &mut notifier_handle,
            );
            assert_that!(ret_val, eq(IOX2_OK));

            iox2_notifier_drop(notifier_handle);
            iox2_port_factory_event_drop(event_service_handle);
            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn targeted_notifications_and_retained_key<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node = create_node::<S>("notifier-key-test");
            let factory = create_event_service(&node, "notifier/key/test");
            let builder = iox2_port_factory_event_notifier_builder(&factory, core::ptr::null_mut());
            let mut notifier = core::ptr::null_mut();
            assert_eq!(
                iox2_port_factory_notifier_builder_create(
                    builder,
                    core::ptr::null_mut(),
                    &mut notifier
                ),
                IOX2_OK
            );

            let mut key_storage = core::mem::MaybeUninit::<iox2_listener_key_t>::uninit();
            let mut visit = VisitContext {
                target_name: "first",
                key_storage: key_storage.as_mut_ptr(),
                key: core::ptr::null_mut(),
                visits: 0,
                notify_result: -1,
                stop: false,
            };
            iox2_notifier_for_each_listener(
                &notifier,
                visit_listener,
                &mut visit as *mut _ as *mut c_void,
            );
            assert_eq!(visit.visits, 0);

            let mut first = listener::<S>(&factory, "first");
            let second = listener::<S>(&factory, "second");
            visit.stop = true;
            iox2_notifier_for_each_listener(
                &notifier,
                visit_listener,
                &mut visit as *mut _ as *mut c_void,
            );
            assert_eq!(visit.visits, 1);
            assert_eq!(visit.notify_result, IOX2_OK);
            assert!(!visit.key.is_null());
            assert_eq!(events(&first), (1, 3));
            assert_eq!(events(&second).0, 0);

            let mut copy_storage = core::mem::MaybeUninit::<iox2_listener_key_t>::uninit();
            let mut copy = core::ptr::null_mut();
            iox2_listener_key_clone(&visit.key, copy_storage.as_mut_ptr(), &mut copy);
            iox2_listener_key_drop(visit.key);
            assert_eq!(
                iox2_notifier_notify_single_listener_with_custom_event_id(
                    &notifier,
                    &copy,
                    &iox2_event_id_t { value: 4 }
                ),
                IOX2_OK
            );
            assert_eq!(events(&first), (1, 4));
            assert_eq!(events(&second).0, 0);
            assert_eq!(
                iox2_notifier_notify_single_listener_with_custom_event_id(
                    &notifier,
                    &copy,
                    &iox2_event_id_t { value: usize::MAX }
                ),
                iox2_notifier_notify_error_e::EVENT_ID_OUT_OF_BOUNDS as i32
            );

            let foreign_factory = create_event_service(&node, "notifier/foreign/key/test");
            let foreign_builder =
                iox2_port_factory_event_notifier_builder(&foreign_factory, core::ptr::null_mut());
            let mut foreign_notifier = core::ptr::null_mut();
            assert_eq!(
                iox2_port_factory_notifier_builder_create(
                    foreign_builder,
                    core::ptr::null_mut(),
                    &mut foreign_notifier
                ),
                IOX2_OK
            );
            assert_eq!(
                iox2_notifier_notify_single_listener(&foreign_notifier, &copy),
                iox2_notifier_notify_error_e::INVALID_LISTENER_KEY as i32
            );
            iox2_notifier_drop(foreign_notifier);
            iox2_port_factory_event_drop(foreign_factory);

            iox2_listener_drop(first);
            first = listener::<S>(&factory, "replacement");
            assert_eq!(
                iox2_notifier_notify_single_listener(&notifier, &copy),
                iox2_notifier_notify_error_e::INVALID_LISTENER_KEY as i32
            );
            assert_eq!(events(&first).0, 0);

            visit.visits = 0;
            visit.target_name = "no-match";
            visit.stop = false;
            visit.key = core::ptr::null_mut();
            iox2_notifier_for_each_listener(
                &notifier,
                visit_listener,
                &mut visit as *mut _ as *mut c_void,
            );
            assert_eq!(visit.visits, 2);
            iox2_listener_key_drop(copy);
            iox2_listener_drop(first);
            iox2_listener_drop(second);
            iox2_notifier_drop(notifier);
            iox2_port_factory_event_drop(factory);
            iox2_node_drop(node);
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
