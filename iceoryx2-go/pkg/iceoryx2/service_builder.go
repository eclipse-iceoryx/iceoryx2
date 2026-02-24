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
*/
import "C"
import (
	"time"
	"unsafe"
)

// ServiceBuilder is used to create or open services.
type ServiceBuilder struct {
	handle      C.iox2_service_builder_h
	serviceType ServiceType
}

// PublishSubscribe returns a ServiceBuilderPubSub for creating publish-subscribe services.
func (b *ServiceBuilder) PublishSubscribe() *ServiceBuilderPubSub {
	if b.handle == nil {
		return nil
	}
	handle := C.iox2_service_builder_pub_sub(b.handle)
	// The original handle is consumed
	b.handle = nil
	return &ServiceBuilderPubSub{
		handle:      handle,
		serviceType: b.serviceType,
	}
}

// Event returns a ServiceBuilderEvent for creating event services.
func (b *ServiceBuilder) Event() *ServiceBuilderEvent {
	if b.handle == nil {
		return nil
	}
	handle := C.iox2_service_builder_event(b.handle)
	// The original handle is consumed
	b.handle = nil
	return &ServiceBuilderEvent{
		handle:      handle,
		serviceType: b.serviceType,
	}
}

// ServiceBuilderPubSub is used to configure and create publish-subscribe services.
type ServiceBuilderPubSub struct {
	handle       C.iox2_service_builder_pub_sub_h
	serviceType  ServiceType
	payloadType  string
	payloadSize  uint64
	payloadAlign uint64
}

// PayloadType sets the payload type details for the service.
// typeName should be a unique identifier for the type (e.g., "MyData").
// size is the size of the payload in bytes.
// alignment is the alignment requirement for the payload.
func (b *ServiceBuilderPubSub) PayloadType(typeName string, size, alignment uint64) *ServiceBuilderPubSub {
	b.payloadType = typeName
	b.payloadSize = size
	b.payloadAlign = alignment
	return b
}

// MaxPublishers sets the maximum number of publishers for this service.
func (b *ServiceBuilderPubSub) MaxPublishers(n uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_max_publishers(&b.handle, C.c_size_t(n))
	}
	return b
}

// MaxSubscribers sets the maximum number of subscribers for this service.
func (b *ServiceBuilderPubSub) MaxSubscribers(n uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_max_subscribers(&b.handle, C.c_size_t(n))
	}
	return b
}

// HistorySize sets the number of samples that are stored for late-joining subscribers.
func (b *ServiceBuilderPubSub) HistorySize(n uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_history_size(&b.handle, C.c_size_t(n))
	}
	return b
}

// SubscriberMaxBufferSize sets the maximum buffer size for subscribers.
func (b *ServiceBuilderPubSub) SubscriberMaxBufferSize(n uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_subscriber_max_buffer_size(&b.handle, C.c_size_t(n))
	}
	return b
}

// EnableSafeOverflow enables safe overflow behavior (oldest samples are discarded when buffer is full).
func (b *ServiceBuilderPubSub) EnableSafeOverflow(enable bool) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_enable_safe_overflow(&b.handle, C.bool(enable))
	}
	return b
}

// MaxNodes sets the maximum number of nodes that can use this service.
func (b *ServiceBuilderPubSub) MaxNodes(n uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_max_nodes(&b.handle, C.c_size_t(n))
	}
	return b
}

// SubscriberMaxBorrowedSamples sets the maximum number of samples a subscriber can borrow at once.
func (b *ServiceBuilderPubSub) SubscriberMaxBorrowedSamples(n uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_subscriber_max_borrowed_samples(&b.handle, C.c_size_t(n))
	}
	return b
}

// PayloadAlignment sets the alignment requirement for payloads.
func (b *ServiceBuilderPubSub) PayloadAlignment(alignment uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		C.iox2_service_builder_pub_sub_set_payload_alignment(&b.handle, C.c_size_t(alignment))
	}
	return b
}

// UserHeaderType sets the user header type details for the service.
// typeName should be a unique identifier for the type.
// size is the size of the user header in bytes.
// alignment is the alignment requirement for the user header.
func (b *ServiceBuilderPubSub) UserHeaderType(typeName string, size, alignment uint64) *ServiceBuilderPubSub {
	if b.handle != nil {
		cTypeName := C.CString(typeName)
		defer C.free(unsafe.Pointer(cTypeName))

		C.iox2_service_builder_pub_sub_set_user_header_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(typeName)),
			C.c_size_t(size),
			C.c_size_t(alignment),
		)
	}
	return b
}

// OpenOrCreate opens an existing service or creates a new one if it doesn't exist.
func (b *ServiceBuilderPubSub) OpenOrCreate() (*PortFactoryPubSub, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	// Set payload type if specified
	if b.payloadType != "" {
		cTypeName := C.CString(b.payloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_pub_sub_set_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.payloadType)),
			C.c_size_t(b.payloadSize),
			C.c_size_t(b.payloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	var serviceHandle C.iox2_port_factory_pub_sub_h
	result := C.iox2_service_builder_pub_sub_open_or_create(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, PubSubOpenOrCreateError(result)
	}

	return &PortFactoryPubSub{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// Open opens an existing service.
func (b *ServiceBuilderPubSub) Open() (*PortFactoryPubSub, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	// Set payload type if specified
	if b.payloadType != "" {
		cTypeName := C.CString(b.payloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_pub_sub_set_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.payloadType)),
			C.c_size_t(b.payloadSize),
			C.c_size_t(b.payloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	var serviceHandle C.iox2_port_factory_pub_sub_h
	result := C.iox2_service_builder_pub_sub_open(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, PubSubOpenOrCreateError(result)
	}

	return &PortFactoryPubSub{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// Create creates a new service (fails if it already exists).
func (b *ServiceBuilderPubSub) Create() (*PortFactoryPubSub, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	// Set payload type if specified
	if b.payloadType != "" {
		cTypeName := C.CString(b.payloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_pub_sub_set_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.payloadType)),
			C.c_size_t(b.payloadSize),
			C.c_size_t(b.payloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	var serviceHandle C.iox2_port_factory_pub_sub_h
	result := C.iox2_service_builder_pub_sub_create(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, PubSubOpenOrCreateError(result)
	}

	return &PortFactoryPubSub{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// ServiceBuilderEvent is used to configure and create event services.
type ServiceBuilderEvent struct {
	handle      C.iox2_service_builder_event_h
	serviceType ServiceType
}

// MaxNotifiers sets the maximum number of notifiers for this service.
func (b *ServiceBuilderEvent) MaxNotifiers(n uint64) *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_set_max_notifiers(&b.handle, C.c_size_t(n))
	}
	return b
}

// MaxListeners sets the maximum number of listeners for this service.
func (b *ServiceBuilderEvent) MaxListeners(n uint64) *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_set_max_listeners(&b.handle, C.c_size_t(n))
	}
	return b
}

// EventIdMaxValue sets the maximum event ID value.
func (b *ServiceBuilderEvent) EventIdMaxValue(n uint64) *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_set_event_id_max_value(&b.handle, C.c_size_t(n))
	}
	return b
}

// MaxNodes sets the maximum number of nodes that can use this event service.
func (b *ServiceBuilderEvent) MaxNodes(n uint64) *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_set_max_nodes(&b.handle, C.c_size_t(n))
	}
	return b
}

// Deadline sets the deadline duration for the event service.
// Listeners must receive events within this duration.
func (b *ServiceBuilderEvent) Deadline(deadline time.Duration) *ServiceBuilderEvent {
	if b.handle != nil {
		secs := C.uint64_t(uint64(deadline.Seconds()))
		nanos := C.uint32_t(uint32(deadline.Nanoseconds() % 1e9))
		C.iox2_service_builder_event_set_deadline(&b.handle, secs, nanos)
	}
	return b
}

// DisableDeadline disables the deadline for the event service.
func (b *ServiceBuilderEvent) DisableDeadline() *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_disable_deadline(&b.handle)
	}
	return b
}

// NotifierDeadEvent sets the event ID that is emitted when a notifier dies.
func (b *ServiceBuilderEvent) NotifierDeadEvent(id uint64) *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_set_notifier_dead_event(&b.handle, C.c_size_t(id))
	}
	return b
}

// DisableNotifierDeadEvent disables the notifier dead event notification.
func (b *ServiceBuilderEvent) DisableNotifierDeadEvent() *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_disable_notifier_dead_event(&b.handle)
	}
	return b
}

// NotifierCreatedEvent sets the event ID that is emitted when a notifier is created.
func (b *ServiceBuilderEvent) NotifierCreatedEvent(id uint64) *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_set_notifier_created_event(&b.handle, C.c_size_t(id))
	}
	return b
}

// DisableNotifierCreatedEvent disables the notifier created event notification.
func (b *ServiceBuilderEvent) DisableNotifierCreatedEvent() *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_disable_notifier_created_event(&b.handle)
	}
	return b
}

// NotifierDroppedEvent sets the event ID that is emitted when a notifier is dropped.
func (b *ServiceBuilderEvent) NotifierDroppedEvent(id uint64) *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_set_notifier_dropped_event(&b.handle, C.c_size_t(id))
	}
	return b
}

// DisableNotifierDroppedEvent disables the notifier dropped event notification.
func (b *ServiceBuilderEvent) DisableNotifierDroppedEvent() *ServiceBuilderEvent {
	if b.handle != nil {
		C.iox2_service_builder_event_disable_notifier_dropped_event(&b.handle)
	}
	return b
}

// OpenOrCreate opens an existing event service or creates a new one if it doesn't exist.
func (b *ServiceBuilderEvent) OpenOrCreate() (*PortFactoryEvent, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var serviceHandle C.iox2_port_factory_event_h
	result := C.iox2_service_builder_event_open_or_create(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, EventOpenOrCreateError(result)
	}

	return &PortFactoryEvent{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// Open opens an existing event service.
func (b *ServiceBuilderEvent) Open() (*PortFactoryEvent, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var serviceHandle C.iox2_port_factory_event_h
	result := C.iox2_service_builder_event_open(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, EventOpenOrCreateError(result)
	}

	return &PortFactoryEvent{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// Create creates a new event service (fails if it already exists).
func (b *ServiceBuilderEvent) Create() (*PortFactoryEvent, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var serviceHandle C.iox2_port_factory_event_h
	result := C.iox2_service_builder_event_create(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, EventOpenOrCreateError(result)
	}

	return &PortFactoryEvent{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// RequestResponse returns a ServiceBuilderRequestResponse for creating request-response services.
func (b *ServiceBuilder) RequestResponse() *ServiceBuilderRequestResponse {
	if b.handle == nil {
		return nil
	}
	handle := C.iox2_service_builder_request_response(b.handle)
	// The original handle is consumed
	b.handle = nil
	return &ServiceBuilderRequestResponse{
		handle:      handle,
		serviceType: b.serviceType,
	}
}

// ServiceBuilderRequestResponse is used to configure and create request-response services.
type ServiceBuilderRequestResponse struct {
	handle               C.iox2_service_builder_request_response_h
	serviceType          ServiceType
	requestPayloadType   string
	requestPayloadSize   uint64
	requestPayloadAlign  uint64
	responsePayloadType  string
	responsePayloadSize  uint64
	responsePayloadAlign uint64
}

// RequestPayloadType sets the request payload type details for the service.
func (b *ServiceBuilderRequestResponse) RequestPayloadType(typeName string, size, alignment uint64) *ServiceBuilderRequestResponse {
	b.requestPayloadType = typeName
	b.requestPayloadSize = size
	b.requestPayloadAlign = alignment
	return b
}

// ResponsePayloadType sets the response payload type details for the service.
func (b *ServiceBuilderRequestResponse) ResponsePayloadType(typeName string, size, alignment uint64) *ServiceBuilderRequestResponse {
	b.responsePayloadType = typeName
	b.responsePayloadSize = size
	b.responsePayloadAlign = alignment
	return b
}

// MaxClients sets the maximum number of clients for this service.
func (b *ServiceBuilderRequestResponse) MaxClients(n uint64) *ServiceBuilderRequestResponse {
	if b.handle != nil {
		C.iox2_service_builder_request_response_max_clients(&b.handle, C.c_size_t(n))
	}
	return b
}

// MaxServers sets the maximum number of servers for this service.
func (b *ServiceBuilderRequestResponse) MaxServers(n uint64) *ServiceBuilderRequestResponse {
	if b.handle != nil {
		C.iox2_service_builder_request_response_max_servers(&b.handle, C.c_size_t(n))
	}
	return b
}

// MaxActiveRequestsPerClient sets the maximum number of active requests per client.
func (b *ServiceBuilderRequestResponse) MaxActiveRequestsPerClient(n uint64) *ServiceBuilderRequestResponse {
	if b.handle != nil {
		C.iox2_service_builder_request_response_max_active_requests_per_client(&b.handle, C.c_size_t(n))
	}
	return b
}

// MaxResponseBufferSize sets the maximum response buffer size.
func (b *ServiceBuilderRequestResponse) MaxResponseBufferSize(n uint64) *ServiceBuilderRequestResponse {
	if b.handle != nil {
		C.iox2_service_builder_request_response_max_response_buffer_size(&b.handle, C.c_size_t(n))
	}
	return b
}

// EnableFireAndForgetRequests enables fire and forget mode for requests.
func (b *ServiceBuilderRequestResponse) EnableFireAndForgetRequests(enable bool) *ServiceBuilderRequestResponse {
	if b.handle != nil {
		C.iox2_service_builder_request_response_enable_fire_and_forget_requests(&b.handle, C.bool(enable))
	}
	return b
}

// EnableSafeOverflowForRequests enables safe overflow for the request queue.
func (b *ServiceBuilderRequestResponse) EnableSafeOverflowForRequests(enable bool) *ServiceBuilderRequestResponse {
	if b.handle != nil {
		C.iox2_service_builder_request_response_enable_safe_overflow_for_requests(&b.handle, C.bool(enable))
	}
	return b
}

// EnableSafeOverflowForResponses enables safe overflow for the response queue.
func (b *ServiceBuilderRequestResponse) EnableSafeOverflowForResponses(enable bool) *ServiceBuilderRequestResponse {
	if b.handle != nil {
		C.iox2_service_builder_request_response_enable_safe_overflow_for_responses(&b.handle, C.bool(enable))
	}
	return b
}

// OpenOrCreate opens an existing request-response service or creates a new one if it doesn't exist.
func (b *ServiceBuilderRequestResponse) OpenOrCreate() (*PortFactoryRequestResponse, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	// Set request payload type if specified
	if b.requestPayloadType != "" {
		cTypeName := C.CString(b.requestPayloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_request_response_set_request_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.requestPayloadType)),
			C.c_size_t(b.requestPayloadSize),
			C.c_size_t(b.requestPayloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	// Set response payload type if specified
	if b.responsePayloadType != "" {
		cTypeName := C.CString(b.responsePayloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_request_response_set_response_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.responsePayloadType)),
			C.c_size_t(b.responsePayloadSize),
			C.c_size_t(b.responsePayloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	var serviceHandle C.iox2_port_factory_request_response_h
	result := C.iox2_service_builder_request_response_open_or_create(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, RequestResponseOpenOrCreateError(result)
	}

	return &PortFactoryRequestResponse{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// Open opens an existing request-response service.
func (b *ServiceBuilderRequestResponse) Open() (*PortFactoryRequestResponse, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	// Set request payload type if specified
	if b.requestPayloadType != "" {
		cTypeName := C.CString(b.requestPayloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_request_response_set_request_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.requestPayloadType)),
			C.c_size_t(b.requestPayloadSize),
			C.c_size_t(b.requestPayloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	// Set response payload type if specified
	if b.responsePayloadType != "" {
		cTypeName := C.CString(b.responsePayloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_request_response_set_response_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.responsePayloadType)),
			C.c_size_t(b.responsePayloadSize),
			C.c_size_t(b.responsePayloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	var serviceHandle C.iox2_port_factory_request_response_h
	result := C.iox2_service_builder_request_response_open(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, RequestResponseOpenOrCreateError(result)
	}

	return &PortFactoryRequestResponse{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}

// Create creates a new request-response service (fails if it already exists).
func (b *ServiceBuilderRequestResponse) Create() (*PortFactoryRequestResponse, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	// Set request payload type if specified
	if b.requestPayloadType != "" {
		cTypeName := C.CString(b.requestPayloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_request_response_set_request_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.requestPayloadType)),
			C.c_size_t(b.requestPayloadSize),
			C.c_size_t(b.requestPayloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	// Set response payload type if specified
	if b.responsePayloadType != "" {
		cTypeName := C.CString(b.responsePayloadType)
		defer C.free(unsafe.Pointer(cTypeName))

		result := C.iox2_service_builder_request_response_set_response_payload_type_details(
			&b.handle,
			C.iox2_type_variant_e_FIXED_SIZE,
			cTypeName,
			C.c_size_t(len(b.responsePayloadType)),
			C.c_size_t(b.responsePayloadSize),
			C.c_size_t(b.responsePayloadAlign),
		)
		if result != C.IOX2_OK {
			return nil, TypeDetailError(result)
		}
	}

	var serviceHandle C.iox2_port_factory_request_response_h
	result := C.iox2_service_builder_request_response_create(b.handle, nil, &serviceHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, RequestResponseOpenOrCreateError(result)
	}

	return &PortFactoryRequestResponse{
		handle:      serviceHandle,
		serviceType: b.serviceType,
	}, nil
}
