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
*/
import "C"
import "unsafe"

// PublishSubscribeHeader contains metadata about a published sample.
type PublishSubscribeHeader struct {
	handle C.iox2_publish_subscribe_header_h
}

// Close releases the resources associated with the header.
func (h *PublishSubscribeHeader) Close() error {
	if h.handle != nil {
		C.iox2_publish_subscribe_header_drop(h.handle)
		h.handle = nil
	}
	return nil
}

// PublisherID returns the unique ID of the publisher that sent the sample.
func (h *PublishSubscribeHeader) PublisherID() (*UniquePublisherId, error) {
	if h.handle == nil {
		return nil, ErrHandleClosed
	}

	var idHandle C.iox2_unique_publisher_id_h
	C.iox2_publish_subscribe_header_publisher_id(&h.handle, nil, &idHandle)

	return &UniquePublisherId{handle: idHandle}, nil
}

// NumberOfElements returns the number of elements in the payload.
// For slices, this is the number of elements. For single values, this is 1.
func (h *PublishSubscribeHeader) NumberOfElements() uint64 {
	if h.handle == nil {
		return 0
	}
	return uint64(C.iox2_publish_subscribe_header_number_of_elements(&h.handle))
}

// UserHeader provides access to custom user-defined header data.
// The returned pointer is valid until the associated sample is closed.
type UserHeader struct {
	ptr  unsafe.Pointer
	size uintptr
}

// Ptr returns the raw pointer to the user header data.
func (h *UserHeader) Ptr() unsafe.Pointer {
	return h.ptr
}

// Size returns the size of the user header in bytes.
func (h *UserHeader) Size() uintptr {
	return h.size
}

// As interprets the user header as a value of type T.
func UserHeaderAs[T any](h *UserHeader) *T {
	if h.ptr == nil {
		return nil
	}
	return (*T)(h.ptr)
}

// UserHeaderMut provides mutable access to custom user-defined header data.
type UserHeaderMut struct {
	ptr  unsafe.Pointer
	size uintptr
}

// Ptr returns the raw pointer to the user header data.
func (h *UserHeaderMut) Ptr() unsafe.Pointer {
	return h.ptr
}

// Size returns the size of the user header in bytes.
func (h *UserHeaderMut) Size() uintptr {
	return h.size
}

// As interprets the user header as a mutable pointer of type T.
func UserHeaderMutAs[T any](h *UserHeaderMut) *T {
	if h.ptr == nil {
		return nil
	}
	return (*T)(h.ptr)
}
