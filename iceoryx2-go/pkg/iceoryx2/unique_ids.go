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
import "encoding/binary"

// UniquePublisherId is a system-wide unique identifier for a publisher.
type UniquePublisherId struct {
	handle C.iox2_unique_publisher_id_h
}

// Close releases the resources associated with the UniquePublisherId.
func (id *UniquePublisherId) Close() error {
	if id.handle != nil {
		C.iox2_unique_publisher_id_drop(id.handle)
		id.handle = nil
	}
	return nil
}

// Value returns the raw bytes of the unique ID.
func (id *UniquePublisherId) Value() uint64 {
	if id.handle == nil {
		return 0
	}
	var buf [8]byte
	C.iox2_unique_publisher_id_value(id.handle, (*C.uint8_t)(&buf[0]), C.size_t(len(buf)))
	return binary.NativeEndian.Uint64(buf[:])
}

// Equals checks if two UniquePublisherIds are equal.
func (id *UniquePublisherId) Equals(other *UniquePublisherId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_publisher_id_eq(&id.handle, &other.handle))
}

// Less checks if this ID is less than another (for ordering).
func (id *UniquePublisherId) Less(other *UniquePublisherId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_publisher_id_less(&id.handle, &other.handle))
}

// UniqueSubscriberId is a system-wide unique identifier for a subscriber.
type UniqueSubscriberId struct {
	handle C.iox2_unique_subscriber_id_h
}

// Close releases the resources associated with the UniqueSubscriberId.
func (id *UniqueSubscriberId) Close() error {
	if id.handle != nil {
		C.iox2_unique_subscriber_id_drop(id.handle)
		id.handle = nil
	}
	return nil
}

// Value returns the raw bytes of the unique ID.
func (id *UniqueSubscriberId) Value() uint64 {
	if id.handle == nil {
		return 0
	}
	var buf [8]byte
	C.iox2_unique_subscriber_id_value(id.handle, (*C.uint8_t)(&buf[0]), C.size_t(len(buf)))
	return binary.NativeEndian.Uint64(buf[:])
}

// Equals checks if two UniqueSubscriberIds are equal.
func (id *UniqueSubscriberId) Equals(other *UniqueSubscriberId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_subscriber_id_eq(&id.handle, &other.handle))
}

// Less checks if this ID is less than another (for ordering).
func (id *UniqueSubscriberId) Less(other *UniqueSubscriberId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_subscriber_id_less(&id.handle, &other.handle))
}

// UniqueListenerId is a system-wide unique identifier for a listener.
type UniqueListenerId struct {
	handle C.iox2_unique_listener_id_h
}

// Close releases the resources associated with the UniqueListenerId.
func (id *UniqueListenerId) Close() error {
	if id.handle != nil {
		C.iox2_unique_listener_id_drop(id.handle)
		id.handle = nil
	}
	return nil
}

// Value returns the raw bytes of the unique ID.
func (id *UniqueListenerId) Value() uint64 {
	if id.handle == nil {
		return 0
	}
	var buf [8]byte
	C.iox2_unique_listener_id_value(id.handle, (*C.uint8_t)(&buf[0]), C.size_t(len(buf)))
	return binary.NativeEndian.Uint64(buf[:])
}

// Equals checks if two UniqueListenerIds are equal.
func (id *UniqueListenerId) Equals(other *UniqueListenerId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_listener_id_eq(&id.handle, &other.handle))
}

// Less checks if this ID is less than another (for ordering).
func (id *UniqueListenerId) Less(other *UniqueListenerId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_listener_id_less(&id.handle, &other.handle))
}

// UniqueNotifierId is a system-wide unique identifier for a notifier.
type UniqueNotifierId struct {
	handle C.iox2_unique_notifier_id_h
}

// Close releases the resources associated with the UniqueNotifierId.
func (id *UniqueNotifierId) Close() error {
	if id.handle != nil {
		C.iox2_unique_notifier_id_drop(id.handle)
		id.handle = nil
	}
	return nil
}

// Value returns the raw bytes of the unique ID.
func (id *UniqueNotifierId) Value() uint64 {
	if id.handle == nil {
		return 0
	}
	var buf [8]byte
	C.iox2_unique_notifier_id_value(id.handle, (*C.uint8_t)(&buf[0]), C.size_t(len(buf)))
	return binary.NativeEndian.Uint64(buf[:])
}

// Equals checks if two UniqueNotifierIds are equal.
func (id *UniqueNotifierId) Equals(other *UniqueNotifierId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_notifier_id_eq(&id.handle, &other.handle))
}

// Less checks if this ID is less than another (for ordering).
func (id *UniqueNotifierId) Less(other *UniqueNotifierId) bool {
	if id.handle == nil || other.handle == nil {
		return false
	}
	return bool(C.iox2_unique_notifier_id_less(&id.handle, &other.handle))
}
