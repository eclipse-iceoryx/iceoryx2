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
	"context"
	"errors"
	"sync"
	"time"
	"unsafe"
)

// PortFactoryPubSub represents an opened publish-subscribe service.
// It is used to create publishers and subscribers.
type PortFactoryPubSub struct {
	handle      C.iox2_port_factory_pub_sub_h
	serviceType ServiceType
}

// Close releases the resources associated with the PortFactoryPubSub.
// Implements io.Closer.
func (p *PortFactoryPubSub) Close() error {
	if p.handle != nil {
		C.iox2_port_factory_pub_sub_drop(p.handle)
		p.handle = nil
	}
	return nil
}

// PublisherBuilder returns a builder for creating a new Publisher.
func (p *PortFactoryPubSub) PublisherBuilder() *PublisherBuilder {
	if p.handle == nil {
		return nil
	}
	handle := C.iox2_port_factory_pub_sub_publisher_builder(&p.handle, nil)
	return &PublisherBuilder{
		handle:      handle,
		serviceType: p.serviceType,
	}
}

// SubscriberBuilder returns a builder for creating a new Subscriber.
func (p *PortFactoryPubSub) SubscriberBuilder() *SubscriberBuilder {
	if p.handle == nil {
		return nil
	}
	handle := C.iox2_port_factory_pub_sub_subscriber_builder(&p.handle, nil)
	return &SubscriberBuilder{
		handle:      handle,
		serviceType: p.serviceType,
	}
}

// Attributes returns the service's attribute set.
// The returned AttributeSet is valid for the lifetime of this PortFactoryPubSub.
func (p *PortFactoryPubSub) Attributes() *AttributeSet {
	if p.handle == nil {
		return nil
	}
	ptr := C.iox2_port_factory_pub_sub_attributes(&p.handle)
	if ptr == nil {
		return nil
	}
	return &AttributeSet{ptr: ptr}
}

// StaticConfig returns the static configuration of the service.
func (p *PortFactoryPubSub) StaticConfig() *StaticConfigPubSub {
	if p.handle == nil {
		return nil
	}

	var cConfig C.iox2_static_config_publish_subscribe_t
	C.iox2_port_factory_pub_sub_static_config(&p.handle, &cConfig)

	return &StaticConfigPubSub{
		MaxSubscribers:               uint64(cConfig.max_subscribers),
		MaxPublishers:                uint64(cConfig.max_publishers),
		MaxNodes:                     uint64(cConfig.max_nodes),
		HistorySize:                  uint64(cConfig.history_size),
		SubscriberMaxBufferSize:      uint64(cConfig.subscriber_max_buffer_size),
		SubscriberMaxBorrowedSamples: uint64(cConfig.subscriber_max_borrowed_samples),
		EnableSafeOverflow:           bool(cConfig.enable_safe_overflow),
	}
}

// NumberOfPublishers returns the number of currently connected publishers.
func (p *PortFactoryPubSub) NumberOfPublishers() uint64 {
	if p.handle == nil {
		return 0
	}
	return uint64(C.iox2_port_factory_pub_sub_dynamic_config_number_of_publishers(&p.handle))
}

// NumberOfSubscribers returns the number of currently connected subscribers.
func (p *PortFactoryPubSub) NumberOfSubscribers() uint64 {
	if p.handle == nil {
		return 0
	}
	return uint64(C.iox2_port_factory_pub_sub_dynamic_config_number_of_subscribers(&p.handle))
}

// ServiceName returns the name of the service.
func (p *PortFactoryPubSub) ServiceName() string {
	if p.handle == nil {
		return ""
	}
	ptr := C.iox2_port_factory_pub_sub_service_name(&p.handle)
	if ptr == nil {
		return ""
	}
	var nameLen C.c_size_t
	cStr := C.iox2_service_name_as_chars(ptr, &nameLen)
	return C.GoStringN(cStr, C.int(nameLen))
}

// ServiceID returns the unique identifier of the service.
func (p *PortFactoryPubSub) ServiceID() string {
	if p.handle == nil {
		return ""
	}
	// IOX2_SERVICE_ID_LENGTH is typically 64; add buffer for safety.
	var buf [128]C.char
	C.iox2_port_factory_pub_sub_service_id(&p.handle, &buf[0], C.size_t(len(buf)))
	return C.GoString(&buf[0])
}

// PublisherBuilder is used to configure and create a Publisher.
type PublisherBuilder struct {
	handle      C.iox2_port_factory_publisher_builder_h
	serviceType ServiceType
}

// MaxSliceLen sets the maximum slice length for loans (for dynamic-sized payloads).
func (b *PublisherBuilder) MaxSliceLen(n uint64) *PublisherBuilder {
	if b.handle != nil {
		C.iox2_port_factory_publisher_builder_set_initial_max_slice_len(&b.handle, C.c_size_t(n))
	}
	return b
}

// UnableToDeliverStrategy sets the strategy when subscriber buffer is full.
func (b *PublisherBuilder) UnableToDeliverStrategy(strategy UnableToDeliverStrategy) *PublisherBuilder {
	if b.handle != nil {
		C.iox2_port_factory_publisher_builder_unable_to_deliver_strategy(&b.handle, uint32(strategy))
	}
	return b
}

// Create creates the Publisher.
func (b *PublisherBuilder) Create() (*Publisher, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var publisherHandle C.iox2_publisher_h
	result := C.iox2_port_factory_publisher_builder_create(b.handle, nil, &publisherHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, PublisherCreateError(result)
	}

	return &Publisher{
		handle:      publisherHandle,
		serviceType: b.serviceType,
	}, nil
}

// Publisher sends samples to subscribers.
type Publisher struct {
	handle      C.iox2_publisher_h
	serviceType ServiceType
}

// Close releases the resources associated with the Publisher.
// Implements io.Closer.
func (p *Publisher) Close() error {
	if p.handle != nil {
		C.iox2_publisher_drop(p.handle)
		p.handle = nil
	}
	return nil
}

// ID returns the unique identifier of this publisher.
func (p *Publisher) ID() (*UniquePublisherId, error) {
	if p.handle == nil {
		return nil, ErrPublisherClosed
	}

	var idHandle C.iox2_unique_publisher_id_h
	C.iox2_publisher_id(&p.handle, nil, &idHandle)

	return &UniquePublisherId{handle: idHandle}, nil
}

// UpdateConnections manually updates the connection tracking.
// This is typically not needed as connections are updated automatically,
// but can be useful in specific scenarios.
func (p *Publisher) UpdateConnections() error {
	if p.handle == nil {
		return ErrPublisherClosed
	}

	result := C.iox2_publisher_update_connections(&p.handle)
	if result != C.IOX2_OK {
		return ConnectionFailure(result)
	}
	return nil
}

// UnableToDeliverStrategy returns the strategy the publisher follows when a sample
// cannot be delivered because the subscriber's buffer is full.
func (p *Publisher) UnableToDeliverStrategy() UnableToDeliverStrategy {
	if p.handle == nil {
		return UnableToDeliverStrategyBlock
	}
	return UnableToDeliverStrategy(C.iox2_publisher_unable_to_deliver_strategy(&p.handle))
}

// InitialMaxSliceLen returns the maximum slice length that can be loaned in one sample.
func (p *Publisher) InitialMaxSliceLen() uint64 {
	if p.handle == nil {
		return 0
	}
	return uint64(C.iox2_publisher_initial_max_slice_len(&p.handle))
}

// LoanUninit loans an uninitialized sample for writing.
// The caller must write to the payload before sending.
func (p *Publisher) LoanUninit() (*SampleMut, error) {
	return p.LoanSliceUninit(1)
}

// LoanSliceUninit loans an uninitialized sample with the specified number of elements.
func (p *Publisher) LoanSliceUninit(len uint64) (*SampleMut, error) {
	if p.handle == nil {
		return nil, ErrPublisherClosed
	}

	var sampleHandle C.iox2_sample_mut_h
	result := C.iox2_publisher_loan_slice_uninit(&p.handle, nil, &sampleHandle, C.c_size_t(len))

	if result != C.IOX2_OK {
		return nil, LoanError(result)
	}

	return &SampleMut{
		handle:      sampleHandle,
		serviceType: p.serviceType,
	}, nil
}

// Send sends the given data directly (copy-based send).
// For zero-copy, use LoanUninit, write to the payload, and call Send on the SampleMut.
func (p *Publisher) Send(data []byte) error {
	sample, err := p.LoanSliceUninit(uint64(len(data)))
	if err != nil {
		return err
	}

	// Copy data to payload
	payload := sample.PayloadMut()
	copy(payload, data)

	return sample.Send()
}

// SampleMut represents a loaned sample that can be written to and sent.
type SampleMut struct {
	handle      C.iox2_sample_mut_h
	serviceType ServiceType
}

// Close releases the sample without sending it.
// Implements io.Closer.
func (s *SampleMut) Close() error {
	if s.handle != nil {
		C.iox2_sample_mut_drop(s.handle)
		s.handle = nil
	}
	return nil
}

// Header returns the publish-subscribe header for this sample.
func (s *SampleMut) Header() (*PublishSubscribeHeader, error) {
	if s.handle == nil {
		return nil, ErrSampleClosed
	}

	var headerHandle C.iox2_publish_subscribe_header_h
	C.iox2_sample_mut_header(&s.handle, nil, &headerHandle)

	return &PublishSubscribeHeader{handle: headerHandle}, nil
}

// UserHeader returns access to the user-defined header data.
// Returns nil if no user header was configured.
func (s *SampleMut) UserHeader() *UserHeaderMut {
	if s.handle == nil {
		return nil
	}

	var headerPtr unsafe.Pointer
	C.iox2_sample_mut_user_header_mut(&s.handle, &headerPtr)

	if headerPtr == nil {
		return nil
	}

	return &UserHeaderMut{ptr: headerPtr}
}

// PayloadMut returns a mutable slice to the payload data.
// The returned slice is valid until Send or Close is called.
func (s *SampleMut) PayloadMut() []byte {
	if s.handle == nil {
		return nil
	}

	var payload unsafe.Pointer
	var payloadLen C.c_size_t

	C.iox2_sample_mut_payload_mut(&s.handle, &payload, &payloadLen)

	if payload == nil || payloadLen == 0 {
		return nil
	}

	// Create a Go slice backed by the C memory
	return unsafe.Slice((*byte)(payload), int(payloadLen))
}

// Write writes the given data to the sample payload.
// This is a convenience method that copies data to the payload.
func (s *SampleMut) Write(data []byte) {
	payload := s.PayloadMut()
	if payload != nil {
		copy(payload, data)
	}
}

// WriteAt writes data to the sample payload at the specified offset.
func (s *SampleMut) WriteAt(data []byte, offset int) {
	payload := s.PayloadMut()
	if payload != nil && offset < len(payload) {
		copy(payload[offset:], data)
	}
}

// Send sends the sample to all subscribers.
// After calling Send, the SampleMut should not be used.
func (s *SampleMut) Send() error {
	if s.handle == nil {
		return ErrSampleClosed
	}

	result := C.iox2_sample_mut_send(s.handle, nil)
	s.handle = nil

	if result != C.IOX2_OK {
		return SendError(result)
	}
	return nil
}

// SubscriberBuilder is used to configure and create a Subscriber.
type SubscriberBuilder struct {
	handle      C.iox2_port_factory_subscriber_builder_h
	serviceType ServiceType
}

// BufferSize sets the buffer size for the subscriber.
func (b *SubscriberBuilder) BufferSize(n uint64) *SubscriberBuilder {
	if b.handle != nil {
		C.iox2_port_factory_subscriber_builder_set_buffer_size(&b.handle, C.c_size_t(n))
	}
	return b
}

// Create creates the Subscriber.
func (b *SubscriberBuilder) Create() (*Subscriber, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var subscriberHandle C.iox2_subscriber_h
	result := C.iox2_port_factory_subscriber_builder_create(b.handle, nil, &subscriberHandle)

	// The builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, SubscriberCreateError(result)
	}

	return &Subscriber{
		handle:      subscriberHandle,
		serviceType: b.serviceType,
	}, nil
}

// Subscriber receives samples from publishers.
// Subscriber is safe for concurrent use; the handle is protected by an RWMutex
// to ensure Close() waits for any in-flight FFI calls to complete.
type Subscriber struct {
	handle      C.iox2_subscriber_h
	serviceType ServiceType
	mu          sync.RWMutex // protects handle during FFI calls
}

// Close releases the resources associated with the Subscriber.
// Close waits for any in-flight FFI calls (e.g., from ReceiveChannel goroutines) to complete.
// Implements io.Closer.
func (s *Subscriber) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.handle != nil {
		C.iox2_subscriber_drop(s.handle)
		s.handle = nil
	}
	return nil
}

// ID returns the unique identifier of this subscriber.
func (s *Subscriber) ID() (*UniqueSubscriberId, error) {
	if s.handle == nil {
		return nil, ErrSubscriberClosed
	}

	var idHandle C.iox2_unique_subscriber_id_h
	C.iox2_subscriber_id(&s.handle, nil, &idHandle)

	return &UniqueSubscriberId{handle: idHandle}, nil
}

// BufferSize returns the buffer size of this subscriber.
func (s *Subscriber) BufferSize() uint64 {
	if s.handle == nil {
		return 0
	}
	return uint64(C.iox2_subscriber_buffer_size(&s.handle))
}

// Receive receives a sample from the subscriber's buffer.
// Receive receives a sample from the publisher.
// Returns ErrNoData if no sample is available.
func (s *Subscriber) Receive() (*Sample, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if s.handle == nil {
		return nil, ErrSubscriberClosed
	}

	var sampleHandle C.iox2_sample_h
	result := C.iox2_subscriber_receive(&s.handle, nil, &sampleHandle)

	if result != C.IOX2_OK {
		return nil, ReceiveError(result)
	}

	if sampleHandle == nil {
		return nil, ErrNoData
	}

	return &Sample{
		handle:      sampleHandle,
		serviceType: s.serviceType,
	}, nil
}

// ReceiveWithContext waits for a sample with context cancellation support.
// This is the idiomatic Go way to receive with cancellation support.
// The pollInterval parameter controls how often the context is checked (default 10ms if 0).
func (s *Subscriber) ReceiveWithContext(ctx context.Context, pollInterval time.Duration) (*Sample, error) {
	const op = "Subscriber.ReceiveWithContext"

	if s.handle == nil {
		return nil, WrapError(op, ErrSubscriberClosed)
	}

	if pollInterval == 0 {
		pollInterval = 10 * time.Millisecond
	}

	// Try once immediately before paying the cost of allocating a ticker.
	if err := ctx.Err(); err != nil {
		return nil, err
	}
	sample, err := s.Receive()
	if !errors.Is(err, ErrNoData) {
		if err != nil {
			return nil, WrapError(op, err)
		}
		return sample, nil
	}

	// No data on the first attempt; create the ticker for the polling loop.
	ticker := time.NewTicker(pollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-ticker.C:
			sample, err := s.Receive()
			if errors.Is(err, ErrNoData) {
				continue
			}
			if err != nil {
				return nil, WrapError(op, err)
			}
			return sample, nil
		}
	}
}

// ReceiveChannel returns a channel that yields samples as they arrive.
// The channel is closed when the context is cancelled or an error occurs.
// This provides idiomatic Go channel-based integration for select statements.
func (s *Subscriber) ReceiveChannel(ctx context.Context) <-chan *Sample {
	ch := make(chan *Sample)
	go func() {
		defer close(ch)
		for {
			sample, err := s.ReceiveWithContext(ctx, 10*time.Millisecond)
			if err != nil {
				return // Context cancelled or error
			}
			select {
			case <-ctx.Done():
				sample.Close()
				return
			case ch <- sample:
			}
		}
	}()
	return ch
}

// Sample represents a received sample from a publisher.
type Sample struct {
	handle      C.iox2_sample_h
	serviceType ServiceType
}

// Close releases the sample.
// Implements io.Closer.
func (s *Sample) Close() error {
	if s.handle != nil {
		C.iox2_sample_drop(s.handle)
		s.handle = nil
	}
	return nil
}

// Header returns the publish-subscribe header for this sample.
func (s *Sample) Header() (*PublishSubscribeHeader, error) {
	if s.handle == nil {
		return nil, ErrSampleClosed
	}

	var headerHandle C.iox2_publish_subscribe_header_h
	C.iox2_sample_header(&s.handle, nil, &headerHandle)

	return &PublishSubscribeHeader{handle: headerHandle}, nil
}

// UserHeader returns access to the user-defined header data.
// Returns nil if no user header was configured.
func (s *Sample) UserHeader() *UserHeader {
	if s.handle == nil {
		return nil
	}

	var headerPtr unsafe.Pointer
	C.iox2_sample_user_header(&s.handle, (*unsafe.Pointer)(unsafe.Pointer(&headerPtr)))

	if headerPtr == nil {
		return nil
	}

	return &UserHeader{ptr: headerPtr}
}

// Payload returns the payload data as a byte slice.
// The returned slice is valid until Close is called.
func (s *Sample) Payload() []byte {
	if s.handle == nil {
		return nil
	}

	var payload unsafe.Pointer
	var payloadLen C.c_size_t

	C.iox2_sample_payload(&s.handle, (*unsafe.Pointer)(unsafe.Pointer(&payload)), &payloadLen)

	if payload == nil || payloadLen == 0 {
		return nil
	}

	// Create a Go slice backed by the C memory (read-only)
	return unsafe.Slice((*byte)(payload), int(payloadLen))
}

// PayloadAs interprets the payload as a value of type T.
// This is a helper for working with fixed-size payloads.
// Note: T must match the actual payload type used on the publisher side.
func PayloadAs[T any](s *Sample) *T {
	return (*T)(s.PayloadPtr())
}

// PayloadPtr returns a raw pointer to the payload data.
// Prefer using PayloadAs[T] for type-safe access.
func (s *Sample) PayloadPtr() unsafe.Pointer {
	if s.handle == nil {
		return nil
	}

	var payload unsafe.Pointer
	var payloadLen C.c_size_t

	C.iox2_sample_payload(&s.handle, (*unsafe.Pointer)(unsafe.Pointer(&payload)), &payloadLen)
	return payload
}

// WritePayloadAs is a helper for writing a value of type T to a SampleMut.
// Note: T must match the payload type configured for the service.
func WritePayloadAs[T any](s *SampleMut, value *T) {
	s.WritePayloadPtr(unsafe.Pointer(value), unsafe.Sizeof(*value))
}

// WritePayloadPtr copies data from src to the sample payload.
// Uses efficient memory copy.
func (s *SampleMut) WritePayloadPtr(src unsafe.Pointer, size uintptr) {
	if s.handle == nil || src == nil {
		return
	}

	var payload unsafe.Pointer
	var payloadLen C.c_size_t

	C.iox2_sample_mut_payload_mut(&s.handle, &payload, &payloadLen)

	if payload != nil && size > 0 {
		C.memcpy(payload, src, C.size_t(size))
	}
}

// PayloadMutAs returns a pointer to the payload as type T.
func PayloadMutAs[T any](s *SampleMut) *T {
	return (*T)(s.PayloadMutPtr())
}

// PayloadMutPtr returns a raw mutable pointer to the payload data.
// Prefer using PayloadMutAs[T] for type-safe access.
func (s *SampleMut) PayloadMutPtr() unsafe.Pointer {
	if s.handle == nil {
		return nil
	}

	var payload unsafe.Pointer
	var payloadLen C.c_size_t

	C.iox2_sample_mut_payload_mut(&s.handle, &payload, &payloadLen)
	return payload
}
