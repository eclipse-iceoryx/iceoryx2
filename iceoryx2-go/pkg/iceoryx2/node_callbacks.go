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

package iceoryx2

/*
#include "iox2/iceoryx2.h"
#include <stdlib.h>
#include <string.h>

// Forward declaration of Go callback - must match the //export signature exactly
// Go int -> C GoInt (which is typically long long or int64)
// Go unsafe.Pointer -> C void*
extern long long goNodeListDispatch(
    int state,
    void* nodeIdPtr,
    char* executable,
    void* nodeNamePtr,
    void* configPtr,
    void* ctx
);

// C trampoline that matches iox2_node_list_callback signature and calls Go
static iox2_callback_progression_e node_list_callback_trampoline(
    iox2_node_state_e state,
    iox2_node_id_ptr node_id_ptr,
    const char* executable,
    iox2_node_name_ptr node_name_ptr,
    iox2_config_ptr config_ptr,
    iox2_callback_context ctx
) {
    long long result = goNodeListDispatch(
        (int)state,
        (void*)node_id_ptr,
        (char*)executable,
        (void*)node_name_ptr,
        (void*)config_ptr,
        ctx
    );
    return (iox2_callback_progression_e)result;
}

// Wrapper function to call iox2_node_list with our trampoline
static int iox2_node_list_with_trampoline(
    iox2_service_type_e service_type,
    iox2_config_ptr config_ptr,
    void* callback_ctx
) {
    return iox2_node_list(service_type, config_ptr, node_list_callback_trampoline, callback_ctx);
}
*/
import "C"
import (
	"runtime/cgo"
	"unsafe"
)

//export goNodeListDispatch
func goNodeListDispatch(
	state C.int,
	nodeIdPtr unsafe.Pointer,
	executable *C.char,
	nodeNamePtr unsafe.Pointer,
	configPtr unsafe.Pointer,
	ctx unsafe.Pointer,
) C.longlong {
	// Retrieve the callback using cgo.Handle
	h := cgo.Handle(uintptr(ctx))
	callback, ok := h.Value().(NodeListCallback)
	if !ok {
		return C.longlong(C.iox2_callback_progression_e_STOP)
	}

	// Extract node name if available
	var name string
	if nodeNamePtr != nil {
		var nameLen C.c_size_t
		cStr := C.iox2_node_name_as_chars(C.iox2_node_name_ptr(nodeNamePtr), &nameLen)
		if cStr != nil {
			name = C.GoStringN(cStr, C.int(nameLen))
		}
	}

	// Create a temporary NodeId wrapper for the callback
	nodeId := &NodeId{ptr: C.iox2_node_id_ptr(nodeIdPtr)}

	result := callback(NodeState(state), nodeId, name)
	return C.longlong(result)
}

// listNodesImpl is the actual implementation using the C callback trampoline.
func listNodesImpl(serviceType ServiceType, config *Config, callback NodeListCallback) error {
	if config == nil || config.ptr == nil {
		return ErrNilHandle
	}

	// Create a cgo.Handle to safely pass the callback through C.
	// cgo.Handle prevents garbage collection until Delete() is called.
	h := cgo.NewHandle(callback)
	defer h.Delete()

	// Store handle value to pass through C.
	// SAFETY: cgo.Handle is designed to be passed through C as void*.
	// The handle prevents garbage collection of the callback until Delete().
	hVal := uintptr(h)

	result := C.iox2_node_list_with_trampoline(
		C.iox2_service_type_e(serviceType),
		config.ptr,
		// #nosec G103 - intentional use of unsafe.Pointer for cgo callback context
		*(*unsafe.Pointer)(unsafe.Pointer(&hVal)),
	)

	if result != C.IOX2_OK {
		return NodeListError(result)
	}
	return nil
}
