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

// Attribute represents a key-value pair for service metadata.
type Attribute struct {
	Key   string
	Value string
}

// AttributeSet is an immutable collection of attributes associated with a service.
type AttributeSet struct {
	ptr C.iox2_attribute_set_ptr
}

// Len returns the number of attributes in the set.
func (a *AttributeSet) Len() uint64 {
	if a.ptr == nil {
		return 0
	}
	return uint64(C.iox2_attribute_set_number_of_attributes(a.ptr))
}

// Get returns all values associated with a key.
func (a *AttributeSet) Get(key string) []string {
	if a.ptr == nil {
		return nil
	}

	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))

	var values []string

	// Use callback to collect all values for this key
	// Note: The C API uses callbacks, so we iterate through all attributes
	for i := uint64(0); i < a.Len(); i++ {
		attr := a.At(i)
		if attr != nil && attr.Key == key {
			values = append(values, attr.Value)
		}
	}

	return values
}

// At returns the attribute at the specified index.
func (a *AttributeSet) At(index uint64) *Attribute {
	if a.ptr == nil || index >= a.Len() {
		return nil
	}

	attrRef := C.iox2_attribute_set_index(a.ptr, C.size_t(index))
	if attrRef == nil {
		return nil
	}

	// Get key
	keyLen := C.iox2_attribute_key_len(attrRef)
	keyBuf := make([]byte, keyLen+1)
	C.iox2_attribute_key(attrRef, (*C.char)(unsafe.Pointer(&keyBuf[0])), C.size_t(len(keyBuf)))

	// Get value
	valueLen := C.iox2_attribute_value_len(attrRef)
	valueBuf := make([]byte, valueLen+1)
	C.iox2_attribute_value(attrRef, (*C.char)(unsafe.Pointer(&valueBuf[0])), C.size_t(len(valueBuf)))

	return &Attribute{
		Key:   string(keyBuf[:keyLen]),
		Value: string(valueBuf[:valueLen]),
	}
}

// All returns all attributes in the set.
func (a *AttributeSet) All() []Attribute {
	if a.ptr == nil {
		return nil
	}

	count := a.Len()
	attrs := make([]Attribute, 0, count)

	for i := uint64(0); i < count; i++ {
		if attr := a.At(i); attr != nil {
			attrs = append(attrs, *attr)
		}
	}

	return attrs
}

// AttributeSpecifier is used to define attributes when creating a service.
type AttributeSpecifier struct {
	handle C.iox2_attribute_specifier_h
}

// NewAttributeSpecifier creates a new AttributeSpecifier.
func NewAttributeSpecifier() (*AttributeSpecifier, error) {
	var handle C.iox2_attribute_specifier_h
	result := C.iox2_attribute_specifier_new(nil, &handle)
	if result != C.IOX2_OK {
		return nil, AttributeDefinitionError(result)
	}
	return &AttributeSpecifier{handle: handle}, nil
}

// Close releases the resources associated with the AttributeSpecifier.
func (a *AttributeSpecifier) Close() error {
	if a.handle != nil {
		C.iox2_attribute_specifier_drop(a.handle)
		a.handle = nil
	}
	return nil
}

// Define adds a key-value attribute.
func (a *AttributeSpecifier) Define(key, value string) error {
	if a.handle == nil {
		return ErrHandleClosed
	}

	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))
	cValue := C.CString(value)
	defer C.free(unsafe.Pointer(cValue))

	result := C.iox2_attribute_specifier_define(&a.handle, cKey, cValue)
	if result != C.IOX2_OK {
		return AttributeDefinitionError(result)
	}
	return nil
}

// AttributeVerifier is used to verify attributes when opening a service.
type AttributeVerifier struct {
	handle C.iox2_attribute_verifier_h
}

// NewAttributeVerifier creates a new AttributeVerifier.
func NewAttributeVerifier() (*AttributeVerifier, error) {
	var handle C.iox2_attribute_verifier_h
	result := C.iox2_attribute_verifier_new(nil, &handle)
	if result != C.IOX2_OK {
		return nil, AttributeVerificationError(result)
	}
	return &AttributeVerifier{handle: handle}, nil
}

// Close releases the resources associated with the AttributeVerifier.
func (a *AttributeVerifier) Close() error {
	if a.handle != nil {
		C.iox2_attribute_verifier_drop(a.handle)
		a.handle = nil
	}
	return nil
}

// Require specifies that a key must have a specific value.
func (a *AttributeVerifier) Require(key, value string) error {
	if a.handle == nil {
		return ErrHandleClosed
	}

	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))
	cValue := C.CString(value)
	defer C.free(unsafe.Pointer(cValue))

	result := C.iox2_attribute_verifier_require(&a.handle, cKey, cValue)
	if result != C.IOX2_OK {
		return AttributeDefinitionError(result)
	}
	return nil
}

// RequireKey specifies that a key must exist (any value).
func (a *AttributeVerifier) RequireKey(key string) error {
	if a.handle == nil {
		return ErrHandleClosed
	}

	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))

	result := C.iox2_attribute_verifier_require_key(&a.handle, cKey)
	if result != C.IOX2_OK {
		return AttributeDefinitionError(result)
	}
	return nil
}
