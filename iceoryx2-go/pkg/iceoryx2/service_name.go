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
*/
import "C"
import "unsafe"

// ServiceName represents a unique identifier for a service.
// It follows a path-like naming convention (e.g., "My/Funk/ServiceName").
type ServiceName struct {
	handle C.iox2_service_name_h
}

// NewServiceName creates a new ServiceName from a string.
// The name must follow a valid path-like format with "/" as separator.
// Maximum length is defined by ServiceNameMaxLength.
func NewServiceName(name string) (*ServiceName, error) {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	var handle C.iox2_service_name_h
	result := C.iox2_service_name_new(nil, cName, C.c_size_t(len(name)), &handle)
	if result != C.IOX2_OK {
		return nil, SemanticStringError(result)
	}

	return &ServiceName{handle: handle}, nil
}

// Close releases the resources associated with the ServiceName.
// After calling Close, the ServiceName should not be used.
// Implements io.Closer.
func (s *ServiceName) Close() error {
	if s.handle != nil {
		C.iox2_service_name_drop(s.handle)
		s.handle = nil
	}
	return nil
}

// String returns the string representation of the ServiceName.
func (s *ServiceName) String() string {
	if s.handle == nil {
		return ""
	}
	ptr := C.iox2_cast_service_name_ptr(s.handle)
	var nameLen C.c_size_t
	cStr := C.iox2_service_name_as_chars(ptr, &nameLen)
	return C.GoStringN(cStr, C.int(nameLen))
}

// ptr returns the C pointer to the service name for internal use.
func (s *ServiceName) ptr() C.iox2_service_name_ptr {
	return C.iox2_cast_service_name_ptr(s.handle)
}

// NodeName represents a name for a node.
type NodeName struct {
	handle C.iox2_node_name_h
}

// NewNodeName creates a new NodeName from a string.
// Maximum length is defined by NodeNameMaxLength.
func NewNodeName(name string) (*NodeName, error) {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	var handle C.iox2_node_name_h
	result := C.iox2_node_name_new(nil, cName, C.c_size_t(len(name)), &handle)
	if result != C.IOX2_OK {
		return nil, SemanticStringError(result)
	}

	return &NodeName{handle: handle}, nil
}

// Close releases the resources associated with the NodeName.
// Implements io.Closer.
func (n *NodeName) Close() error {
	if n.handle != nil {
		C.iox2_node_name_drop(n.handle)
		n.handle = nil
	}
	return nil
}

// String returns the string representation of the NodeName.
func (n *NodeName) String() string {
	if n.handle == nil {
		return ""
	}
	ptr := C.iox2_cast_node_name_ptr(n.handle)
	var nameLen C.c_size_t
	cStr := C.iox2_node_name_as_chars(ptr, &nameLen)
	return C.GoStringN(cStr, C.int(nameLen))
}
