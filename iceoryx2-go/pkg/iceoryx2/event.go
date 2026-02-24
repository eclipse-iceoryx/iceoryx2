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
import (
	"context"
	"errors"
	"sync"
	"time"
)

// PortFactoryEvent represents an opened event service.
// It is used to create notifiers and listeners.
type PortFactoryEvent struct {
	handle      C.iox2_port_factory_event_h
	serviceType ServiceType
}

// Close releases the resources associated with the PortFactoryEvent.
// Implements io.Closer.
func (p *PortFactoryEvent) Close() error {
	if p.handle != nil {
		C.iox2_port_factory_event_drop(p.handle)
		p.handle = nil
	}
	return nil
}

// NotifierBuilder returns a builder for creating a new Notifier.
func (p *PortFactoryEvent) NotifierBuilder() *NotifierBuilder {
	if p.handle == nil {
		return nil
	}
	handle := C.iox2_port_factory_event_notifier_builder(&p.handle, nil)
	return &NotifierBuilder{
		handle:      handle,
		serviceType: p.serviceType,
	}
}

// ListenerBuilder returns a builder for creating a new Listener.
func (p *PortFactoryEvent) ListenerBuilder() *ListenerBuilder {
	if p.handle == nil {
		return nil
	}
	handle := C.iox2_port_factory_event_listener_builder(&p.handle, nil)
	return &ListenerBuilder{
		handle:      handle,
		serviceType: p.serviceType,
	}
}

// Attributes returns the service's attribute set.
// The returned AttributeSet is valid for the lifetime of this PortFactoryEvent.
func (p *PortFactoryEvent) Attributes() *AttributeSet {
	if p.handle == nil {
		return nil
	}
	ptr := C.iox2_port_factory_event_attributes(&p.handle)
	if ptr == nil {
		return nil
	}
	return &AttributeSet{ptr: ptr}
}

// StaticConfig returns the static configuration of the event service.
func (p *PortFactoryEvent) StaticConfig() *StaticConfigEvent {
	if p.handle == nil {
		return nil
	}

	var cConfig C.iox2_static_config_event_t
	C.iox2_port_factory_event_static_config(&p.handle, &cConfig)

	return &StaticConfigEvent{
		MaxListeners:    uint64(cConfig.max_listeners),
		MaxNotifiers:    uint64(cConfig.max_notifiers),
		MaxNodes:        uint64(cConfig.max_nodes),
		EventIdMaxValue: uint64(cConfig.event_id_max_value),
	}
}

// NumberOfNotifiers returns the number of currently connected notifiers.
func (p *PortFactoryEvent) NumberOfNotifiers() uint64 {
	if p.handle == nil {
		return 0
	}
	return uint64(C.iox2_port_factory_event_dynamic_config_number_of_notifiers(&p.handle))
}

// NumberOfListeners returns the number of currently connected listeners.
func (p *PortFactoryEvent) NumberOfListeners() uint64 {
	if p.handle == nil {
		return 0
	}
	return uint64(C.iox2_port_factory_event_dynamic_config_number_of_listeners(&p.handle))
}

// ServiceName returns the name of this event service.
func (p *PortFactoryEvent) ServiceName() string {
	if p.handle == nil {
		return ""
	}
	namePtr := C.iox2_port_factory_event_service_name(&p.handle)
	if namePtr == nil {
		return ""
	}
	var nameLen C.c_size_t
	cStr := C.iox2_service_name_as_chars(namePtr, &nameLen)
	return C.GoStringN(cStr, C.int(nameLen))
}

// ServiceID returns the unique identifier of this event service.
func (p *PortFactoryEvent) ServiceID() string {
	if p.handle == nil {
		return ""
	}
	// IOX2_SERVICE_ID_LENGTH is typically 64, add some buffer
	var buffer [128]C.char
	C.iox2_port_factory_event_service_id(&p.handle, &buffer[0], C.size_t(len(buffer)))
	return C.GoString(&buffer[0])
}

// NotifierBuilder is used to configure and create a Notifier.
type NotifierBuilder struct {
	handle      C.iox2_port_factory_notifier_builder_h
	serviceType ServiceType
}

// DefaultEventId sets the default event ID for notifications.
func (b *NotifierBuilder) DefaultEventId(id uint64) *NotifierBuilder {
	if b.handle != nil {
		eventId := C.iox2_event_id_t{value: C.size_t(id)}
		C.iox2_port_factory_notifier_builder_set_default_event_id(&b.handle, &eventId)
	}
	return b
}

// Create creates the Notifier.
func (b *NotifierBuilder) Create() (*Notifier, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var notifierHandle C.iox2_notifier_h
	result := C.iox2_port_factory_notifier_builder_create(b.handle, nil, &notifierHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, NotifierCreateError(result)
	}

	return &Notifier{
		handle:      notifierHandle,
		serviceType: b.serviceType,
	}, nil
}

// Notifier sends event notifications to listeners.
type Notifier struct {
	handle      C.iox2_notifier_h
	serviceType ServiceType
}

// Close releases the resources associated with the Notifier.
// Implements io.Closer.
func (n *Notifier) Close() error {
	if n.handle != nil {
		C.iox2_notifier_drop(n.handle)
		n.handle = nil
	}
	return nil
}

// ID returns the unique identifier of this notifier.
func (n *Notifier) ID() (*UniqueNotifierId, error) {
	if n.handle == nil {
		return nil, ErrNotifierClosed
	}

	var idHandle C.iox2_unique_notifier_id_h
	C.iox2_notifier_id(&n.handle, nil, &idHandle)

	return &UniqueNotifierId{handle: idHandle}, nil
}

// Deadline returns the deadline duration for this notifier, if configured.
// Returns nil if no deadline is set.
func (n *Notifier) Deadline() *time.Duration {
	if n.handle == nil {
		return nil
	}

	var secs C.uint64_t
	var nanos C.uint32_t

	hasDeadline := C.iox2_notifier_deadline(&n.handle, &secs, &nanos)
	if !bool(hasDeadline) {
		return nil
	}

	d := time.Duration(secs)*time.Second + time.Duration(nanos)*time.Nanosecond
	return &d
}

// Notify sends a notification with the default event ID.
// Returns the number of listeners that were notified.
func (n *Notifier) Notify() (uint64, error) {
	if n.handle == nil {
		return 0, ErrNotifierClosed
	}

	var numberOfListeners C.c_size_t
	result := C.iox2_notifier_notify(&n.handle, &numberOfListeners)

	if result != C.IOX2_OK {
		return 0, NotifierNotifyError(result)
	}
	return uint64(numberOfListeners), nil
}

// NotifyWithEventId sends a notification with a specific event ID.
// Returns the number of listeners that were notified.
func (n *Notifier) NotifyWithEventId(eventId uint64) (uint64, error) {
	if n.handle == nil {
		return 0, ErrNotifierClosed
	}

	cEventId := C.iox2_event_id_t{value: C.size_t(eventId)}
	var numberOfListeners C.c_size_t
	result := C.iox2_notifier_notify_with_custom_event_id(&n.handle, &cEventId, &numberOfListeners)

	if result != C.IOX2_OK {
		return 0, NotifierNotifyError(result)
	}
	return uint64(numberOfListeners), nil
}

// ListenerBuilder is used to configure and create a Listener.
type ListenerBuilder struct {
	handle      C.iox2_port_factory_listener_builder_h
	serviceType ServiceType
}

// Create creates the Listener.
func (b *ListenerBuilder) Create() (*Listener, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var listenerHandle C.iox2_listener_h
	result := C.iox2_port_factory_listener_builder_create(b.handle, nil, &listenerHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, ListenerCreateError(result)
	}

	return &Listener{
		handle:      listenerHandle,
		serviceType: b.serviceType,
	}, nil
}

// Listener receives event notifications from notifiers.
// Listener is safe for concurrent use; the handle is protected by an RWMutex
// to ensure Close() waits for any in-flight FFI calls to complete.
type Listener struct {
	handle      C.iox2_listener_h
	serviceType ServiceType
	mu          sync.RWMutex // protects handle during FFI calls
}

// Close releases the resources associated with the Listener.
// Close waits for any in-flight FFI calls (e.g., from EventChannel goroutines) to complete.
// Implements io.Closer.
func (l *Listener) Close() error {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.handle != nil {
		C.iox2_listener_drop(l.handle)
		l.handle = nil
	}
	return nil
}

// ID returns the unique identifier of this listener.
func (l *Listener) ID() (*UniqueListenerId, error) {
	if l.handle == nil {
		return nil, ErrListenerClosed
	}

	var idHandle C.iox2_unique_listener_id_h
	C.iox2_listener_id(&l.handle, nil, &idHandle)

	return &UniqueListenerId{handle: idHandle}, nil
}

// Deadline returns the deadline duration for this listener, if configured.
// Returns nil if no deadline is set.
func (l *Listener) Deadline() *time.Duration {
	if l.handle == nil {
		return nil
	}

	var secs C.uint64_t
	var nanos C.uint32_t

	hasDeadline := C.iox2_listener_deadline(&l.handle, &secs, &nanos)
	if !bool(hasDeadline) {
		return nil
	}

	d := time.Duration(secs)*time.Second + time.Duration(nanos)*time.Nanosecond
	return &d
}

// TryWaitOne tries to receive a single event without blocking.
// TryWaitOne attempts to receive a single event without blocking.
// Returns ErrNoData if no event is available.
func (l *Listener) TryWaitOne() (*EventId, error) {
	l.mu.RLock()
	defer l.mu.RUnlock()

	if l.handle == nil {
		return nil, ErrListenerClosed
	}

	var hasReceived C.bool
	var eventId C.iox2_event_id_t

	result := C.iox2_listener_try_wait_one(&l.handle, &eventId, &hasReceived)
	if result != C.IOX2_OK {
		return nil, ListenerWaitError(result)
	}

	if !bool(hasReceived) {
		return nil, ErrNoData
	}

	eventValue := EventId(eventId.value)
	return &eventValue, nil
}

// WaitOne waits for a single event with context support.
// This is the idiomatic Go way to wait with cancellation support.
func (l *Listener) WaitOne(ctx context.Context) (*EventId, error) {
	if l.handle == nil {
		return nil, ErrListenerClosed
	}

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
			// Use native timed wait with short timeout to allow context checking
			event, err := l.timedWaitOne(10 * time.Millisecond)
			if errors.Is(err, ErrNoData) {
				// No event yet, continue waiting
				continue
			}
			if err != nil {
				return nil, err
			}
			return event, nil
		}
	}
}

// timedWaitOne waits for a single event with a timeout (internal use).
func (l *Listener) timedWaitOne(timeout time.Duration) (*EventId, error) {
	l.mu.RLock()
	defer l.mu.RUnlock()

	if l.handle == nil {
		return nil, ErrListenerClosed
	}

	var hasReceived C.bool
	var eventId C.iox2_event_id_t

	secs := uint64(timeout.Seconds())
	nanos := uint32(timeout.Nanoseconds() % 1e9)

	result := C.iox2_listener_timed_wait_one(&l.handle, &eventId, &hasReceived, C.uint64_t(secs), C.uint32_t(nanos))
	if result != C.IOX2_OK {
		return nil, ListenerWaitError(result)
	}

	if !bool(hasReceived) {
		return nil, ErrNoData
	}

	eventValue := EventId(eventId.value)
	return &eventValue, nil
}

// TryWaitAll receives all pending events and returns them as a slice.
// Returns an empty slice if no events are available.
func (l *Listener) TryWaitAll() ([]EventId, error) {
	var events []EventId
	for {
		event, err := l.TryWaitOne()
		if errors.Is(err, ErrNoData) {
			break
		}
		if err != nil {
			return events, err
		}
		events = append(events, *event)
	}
	return events, nil
}

// WaitAll waits for at least one event and returns all pending events with context support.
// This is the idiomatic Go way to wait with cancellation support.
func (l *Listener) WaitAll(ctx context.Context) ([]EventId, error) {
	if l.handle == nil {
		return nil, ErrListenerClosed
	}

	// First, wait for at least one event
	firstEvent, err := l.WaitOne(ctx)
	if err != nil {
		return nil, err
	}

	// Collect all remaining pending events
	events := []EventId{*firstEvent}
	for {
		event, err := l.TryWaitOne()
		if errors.Is(err, ErrNoData) {
			break
		}
		if err != nil {
			return events, err
		}
		events = append(events, *event)
	}

	return events, nil
}

// EventChannel returns a channel that yields events as they arrive.
// The channel is closed when the context is cancelled or an error occurs.
// This provides idiomatic Go channel-based integration for select statements.
func (l *Listener) EventChannel(ctx context.Context) <-chan EventId {
	ch := make(chan EventId)
	go func() {
		defer close(ch)
		for {
			event, err := l.WaitOne(ctx)
			if err != nil {
				return // Context cancelled or error
			}
			select {
			case <-ctx.Done():
				return
			case ch <- *event:
			}
		}
	}()
	return ch
}

// TimedWaitAll waits for at least one event with a timeout and returns all pending events.
func (l *Listener) TimedWaitAll(timeout time.Duration) ([]EventId, error) {
	if l.handle == nil {
		return nil, ErrListenerClosed
	}

	// First, wait for at least one event
	firstEvent, err := l.timedWaitOne(timeout)
	if err != nil {
		return nil, err
	}

	// Collect all remaining pending events
	events := []EventId{*firstEvent}
	for {
		event, err := l.TryWaitOne()
		if errors.Is(err, ErrNoData) {
			break
		}
		if err != nil {
			return events, err
		}
		events = append(events, *event)
	}

	return events, nil
}
