//// Copyright (c) 2025 Contributors to the Eclipse Foundation
////
//// See the NOTICE file(s) distributed with this work for additional
//// information regarding copyright ownership.
////
//// This program and the accompanying materials are made available under the
//// terms of the Apache Software License 2.0 which is available at
//// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
//// which is available at https://opensource.org/licenses/MIT.
////
//// SPDX-License-Identifier: Apache-2.0 OR MIT

//#![allow(non_camel_case_types)]

//use crate::api::{
//iox2_service_type_e, iox2_unique_reader_id_h, iox2_unique_reader_id_t, AssertNonNullHandle,
//HandleToType, KeyFfi,
//};
//use core::mem::ManuallyDrop;
//use iceoryx2::port::reader::Reader;
//use iceoryx2_bb_elementary::static_assert::*;
//use iceoryx2_ffi_macros::iceoryx2_ffi;

//// BEGIN types definition

//pub(super) union ReaderUnion {
//ipc: ManuallyDrop<Reader<crate::IpcService, KeyFfi>>,
//local: ManuallyDrop<Reader<crate::LocalService, KeyFfi>>,
//}

//impl ReaderUnion {
//pub(super) fn new_ipc(reader: Reader<crate::IpcService, KeyFfi>) -> Self {
//Self {
//ipc: ManuallyDrop::new(reader),
//}
//}
//pub(super) fn new_local(reader: Reader<crate::LocalService, KeyFfi>) -> Self {
//Self {
//local: ManuallyDrop::new(reader),
//}
//}
//}

//#[repr(C)]
//#[repr(align(16))] // alignment of Option<ReaderUnion>
//pub struct iox2_reader_storage_t {
//// TODO: adapt size and alignment
//internal: [u8; 1232], // magic number obtained with size_of::<Option<ReaderUnion>>()
//}

//#[repr(C)]
//#[iceoryx2_ffi(ReaderUnion)]
//pub struct iox2_reader_t {
//service_type: iox2_service_type_e,
//value: iox2_reader_storage_t,
//deleter: fn(*mut iox2_reader_t),
//}

//impl iox2_reader_t {
//pub(super) fn init(
//&mut self,
//service_type: iox2_service_type_e,
//value: ReaderUnion,
//deleter: fn(*mut iox2_reader_t),
//) {
//self.service_type = service_type;
//self.value.init(value);
//self.deleter = deleter;
//}
//}

//pub struct iox2_reader_h_t;
///// The owning handle for `iox2_reader_t`. Passing the handle to an function transfers the ownership.
//pub type iox2_reader_h = *mut iox2_reader_h_t;
///// The non-owning handle for `iox2_reader_t`. Passing the handle to an function does not transfers the ownership.
//pub type iox2_reader_h_ref = *const iox2_reader_h;

//impl AssertNonNullHandle for iox2_reader_h {
//fn assert_non_null(self) {
//debug_assert!(!self.is_null());
//}
//}

//impl AssertNonNullHandle for iox2_reader_h_ref {
//fn assert_non_null(self) {
//debug_assert!(!self.is_null());
//unsafe {
//debug_assert!(!(*self).is_null());
//}
//}
//}

//impl HandleToType for iox2_reader_h {
//type Target = *mut iox2_reader_t;

//fn as_type(self) -> Self::Target {
//self as *mut _ as _
//}
//}

//impl HandleToType for iox2_reader_h_ref {
//type Target = *mut iox2_reader_t;

//fn as_type(self) -> Self::Target {
//unsafe { *self as *mut _ as _ }
//}
//}

//// END type definition

//// BEGIN C API

///// Returns the unique port id of the reader.
/////
///// # Arguments
/////
///// * `reader_handle` obtained by [`iox2_port_factory_reader_builder_create`](crate::iox2_port_factory_reader_builder_create)
///// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_reader_id_t`].
/////   If it is a NULL pointer, the storage will be allocated on the heap.
///// * `id_handle_ptr` valid pointer to a [`iox2_unique_reader_id_h`].
/////
///// # Safety
/////
///// * `reader_handle` is valid, non-null and was obtained via [`iox2_port_factory_reader_builder_create`](crate::iox2_port_factory_reader_builder_create)
///// * `id` is valid and non-null
//#[no_mangle]
//pub unsafe extern "C" fn iox2_reader_id(
//reader_handle: iox2_reader_h_ref,
//id_struct_ptr: *mut iox2_unique_reader_id_t,
//id_handle_ptr: *mut iox2_unique_reader_id_h,
//) {
//reader_handle.assert_non_null();
//debug_assert!(!id_handle_ptr.is_null());

//fn no_op(_: *mut iox2_unique_reader_id_t) {}
//let mut deleter: fn(*mut iox2_unique_reader_id_t) = no_op;
//let mut storage_ptr = id_struct_ptr;
//if id_struct_ptr.is_null() {
//deleter = iox2_unique_reader_id_t::dealloc;
//storage_ptr = iox2_unique_reader_id_t::alloc();
//}
//debug_assert!(!storage_ptr.is_null());

//let reader = &mut *reader_handle.as_type();

//let id = match reader.service_type {
//iox2_service_type_e::IPC => reader.value.as_mut().ipc.id(),
//iox2_service_type_e::LOCAL => reader.value.as_mut().local.id(),
//};

//(*storage_ptr).init(id, deleter);
//*id_handle_ptr = (*storage_ptr).as_handle();
//}

//// TODO: entry method + ReaderHandle
//#[no_mangle]
//pub unsafe extern "C" fn iox2_reader_entry(reader_handle: iox2_reader_h_ref) {
//reader_handle.assert_non_null();
//}
//// END C API
