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
extern long long goServiceListDispatch(
    void* staticConfigPtr,
    void* ctx
);

// C trampoline that matches iox2_service_list_callback signature and calls Go
static iox2_callback_progression_e service_list_callback_trampoline(
    const struct iox2_static_config_t* static_config,
    iox2_callback_context ctx
) {
    long long result = goServiceListDispatch(
        (void*)static_config,
        ctx
    );
    return (iox2_callback_progression_e)result;
}

// Wrapper function to call iox2_service_list with our trampoline
static int iox2_service_list_with_trampoline(
    iox2_service_type_e service_type,
    iox2_config_ptr config_ptr,
    void* callback_ctx
) {
    return iox2_service_list(service_type, config_ptr, service_list_callback_trampoline, callback_ctx);
}
*/
import "C"
import (
	"runtime/cgo"
	"unsafe"
)

// MessagingPattern defines the communication pattern of a service.
type MessagingPattern int

const (
	// MessagingPatternPublishSubscribe is the publish-subscribe pattern.
	MessagingPatternPublishSubscribe MessagingPattern = C.iox2_messaging_pattern_e_PUBLISH_SUBSCRIBE
	// MessagingPatternEvent is the event pattern.
	MessagingPatternEvent MessagingPattern = C.iox2_messaging_pattern_e_EVENT
	// MessagingPatternRequestResponse is the request-response pattern.
	MessagingPatternRequestResponse MessagingPattern = C.iox2_messaging_pattern_e_REQUEST_RESPONSE
	// MessagingPatternBlackboard is the blackboard pattern.
	MessagingPatternBlackboard MessagingPattern = C.iox2_messaging_pattern_e_BLACKBOARD
)

// String returns the string representation of the messaging pattern.
func (p MessagingPattern) String() string {
	switch p {
	case MessagingPatternPublishSubscribe:
		return "PublishSubscribe"
	case MessagingPatternEvent:
		return "Event"
	case MessagingPatternRequestResponse:
		return "RequestResponse"
	case MessagingPatternBlackboard:
		return "Blackboard"
	default:
		return "Unknown"
	}
}

// ServiceInfo contains information about a discovered service.
type ServiceInfo struct {
	// ID is the unique identifier of the service.
	ID string
	// Name is the name of the service.
	Name string
	// MessagingPattern is the messaging pattern of the service.
	MessagingPattern MessagingPattern
}

// ServiceListCallback is the callback type for service listing.
type ServiceListCallback func(info *ServiceInfo) CallbackProgression

//export goServiceListDispatch
func goServiceListDispatch(
	staticConfigPtr unsafe.Pointer,
	ctx unsafe.Pointer,
) C.longlong {
	// Retrieve the callback using cgo.Handle
	h := cgo.Handle(uintptr(ctx))
	callback, ok := h.Value().(ServiceListCallback)
	if !ok {
		return C.longlong(C.iox2_callback_progression_e_STOP)
	}

	// Extract service info from the static config
	config := (*C.iox2_static_config_t)(staticConfigPtr)
	if config == nil {
		return C.longlong(C.iox2_callback_progression_e_STOP)
	}

	info := &ServiceInfo{
		ID:               C.GoString(&config.id[0]),
		Name:             C.GoString(&config.name[0]),
		MessagingPattern: MessagingPattern(config.messaging_pattern),
	}

	result := callback(info)
	return C.longlong(result)
}

// ListServices lists all available services of the specified type.
// The callback is invoked for each discovered service.
func ListServices(serviceType ServiceType, callback ServiceListCallback) error {
	if callback == nil {
		return ErrNilHandle
	}

	// Get the default config
	config := C.iox2_config_global_config()

	// Create a cgo.Handle to safely pass the callback through C.
	h := cgo.NewHandle(callback)
	defer h.Delete()

	// Store handle value to pass through C.
	hVal := uintptr(h)

	result := C.iox2_service_list_with_trampoline(
		C.iox2_service_type_e(serviceType),
		config,
		// #nosec G103 - intentional use of unsafe.Pointer for cgo callback context
		*(*unsafe.Pointer)(unsafe.Pointer(&hVal)),
	)

	if result != C.IOX2_OK {
		return ServiceListError(result)
	}

	return nil
}

// ServiceExists checks if a service with the given name exists.
func ServiceExists(serviceType ServiceType, serviceName *ServiceName, pattern MessagingPattern) (bool, error) {
	if serviceName == nil {
		return false, ErrNilHandle
	}

	var config C.iox2_config_ptr
	config = C.iox2_config_global_config()

	var serviceDetails C.iox2_static_config_t
	var doesExist C.bool

	result := C.iox2_service_details(
		uint32(serviceType),
		serviceName.ptr(),
		config,
		uint32(pattern),
		&serviceDetails,
		&doesExist,
	)

	if result != C.IOX2_OK {
		return false, ServiceDetailsError(result)
	}

	return bool(doesExist), nil
}

// GetServiceDetails retrieves detailed information about a specific service.
func GetServiceDetails(serviceType ServiceType, serviceName *ServiceName, pattern MessagingPattern) (*ServiceInfo, error) {
	if serviceName == nil {
		return nil, ErrNilHandle
	}

	var config C.iox2_config_ptr
	config = C.iox2_config_global_config()

	var serviceDetails C.iox2_static_config_t
	var doesExist C.bool

	result := C.iox2_service_details(
		uint32(serviceType),
		serviceName.ptr(),
		config,
		uint32(pattern),
		&serviceDetails,
		&doesExist,
	)

	if result != C.IOX2_OK {
		return nil, ServiceDetailsError(result)
	}

	if !bool(doesExist) {
		return nil, ErrNoData // Service doesn't exist
	}

	// Convert C strings to Go strings
	id := C.GoString(&serviceDetails.id[0])
	name := C.GoString(&serviceDetails.name[0])

	return &ServiceInfo{
		ID:               id,
		Name:             name,
		MessagingPattern: MessagingPattern(serviceDetails.messaging_pattern),
	}, nil
}

// ServiceDiscovery provides methods for discovering services.
type ServiceDiscovery struct {
	serviceType ServiceType
}

// NewServiceDiscovery creates a new ServiceDiscovery instance.
func NewServiceDiscovery(serviceType ServiceType) *ServiceDiscovery {
	return &ServiceDiscovery{
		serviceType: serviceType,
	}
}

// Exists checks if a service with the given name and pattern exists.
func (sd *ServiceDiscovery) Exists(serviceName *ServiceName, pattern MessagingPattern) (bool, error) {
	return ServiceExists(sd.serviceType, serviceName, pattern)
}

// Details retrieves detailed information about a specific service.
func (sd *ServiceDiscovery) Details(serviceName *ServiceName, pattern MessagingPattern) (*ServiceInfo, error) {
	return GetServiceDetails(sd.serviceType, serviceName, pattern)
}

// FindPubSubService finds a publish-subscribe service by name.
func (sd *ServiceDiscovery) FindPubSubService(name string) (*ServiceInfo, error) {
	serviceName, err := NewServiceName(name)
	if err != nil {
		return nil, err
	}
	defer serviceName.Close()

	return sd.Details(serviceName, MessagingPatternPublishSubscribe)
}

// FindEventService finds an event service by name.
func (sd *ServiceDiscovery) FindEventService(name string) (*ServiceInfo, error) {
	serviceName, err := NewServiceName(name)
	if err != nil {
		return nil, err
	}
	defer serviceName.Close()

	return sd.Details(serviceName, MessagingPatternEvent)
}

// FindRequestResponseService finds a request-response service by name.
func (sd *ServiceDiscovery) FindRequestResponseService(name string) (*ServiceInfo, error) {
	serviceName, err := NewServiceName(name)
	if err != nil {
		return nil, err
	}
	defer serviceName.Close()

	return sd.Details(serviceName, MessagingPatternRequestResponse)
}

// CollectServices collects all services of a specific type and returns them as a slice.
// This is a convenience wrapper around ListServices.
func CollectServices(serviceType ServiceType) ([]*ServiceInfo, error) {
	var services []*ServiceInfo

	err := ListServices(serviceType, func(info *ServiceInfo) CallbackProgression {
		services = append(services, info)
		return CallbackProgressionContinue
	})
	if err != nil {
		return nil, err
	}

	return services, nil
}

// GlobalConfig returns the global iceoryx2 configuration.
// The returned Config holds a borrowed pointer and must not be closed.
func GlobalConfig() *Config {
	return &Config{ptr: C.iox2_config_global_config()}
}
