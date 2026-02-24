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

// PortFactoryRequestResponse is the factory for creating clients and servers
// in a request-response service.
type PortFactoryRequestResponse struct {
	handle      C.iox2_port_factory_request_response_h
	serviceType ServiceType
}

// Client returns a ClientBuilder for creating a client.
func (p *PortFactoryRequestResponse) Client() *ClientBuilder {
	if p.handle == nil {
		return nil
	}
	handle := C.iox2_port_factory_request_response_client_builder(&p.handle, nil)
	return &ClientBuilder{
		handle:      handle,
		serviceType: p.serviceType,
	}
}

// Server returns a ServerBuilder for creating a server.
func (p *PortFactoryRequestResponse) Server() *ServerBuilder {
	if p.handle == nil {
		return nil
	}
	handle := C.iox2_port_factory_request_response_server_builder(&p.handle, nil)
	return &ServerBuilder{
		handle:      handle,
		serviceType: p.serviceType,
	}
}

// Close releases the port factory resources.
// Implements io.Closer interface.
func (p *PortFactoryRequestResponse) Close() error {
	if p.handle != nil {
		C.iox2_port_factory_request_response_drop(p.handle)
		p.handle = nil
	}
	return nil
}

// ClientBuilder is used to configure and create a Client.
type ClientBuilder struct {
	handle      C.iox2_port_factory_client_builder_h
	serviceType ServiceType
}

// InitialMaxSliceLen sets the initial maximum slice length for loan operations.
func (b *ClientBuilder) InitialMaxSliceLen(len uint64) *ClientBuilder {
	if b.handle != nil {
		C.iox2_port_factory_client_builder_set_initial_max_slice_len(&b.handle, C.c_size_t(len))
	}
	return b
}

// AllocationStrategy sets the allocation strategy for the client.
func (b *ClientBuilder) AllocationStrategy(strategy AllocationStrategy) *ClientBuilder {
	if b.handle != nil {
		C.iox2_port_factory_client_builder_set_allocation_strategy(&b.handle, uint32(strategy))
	}
	return b
}

// Create creates a new Client.
func (b *ClientBuilder) Create() (*Client, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var clientHandle C.iox2_client_h
	result := C.iox2_port_factory_client_builder_create(b.handle, nil, &clientHandle)

	// Builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, ClientCreateError(result)
	}

	return &Client{
		handle:      clientHandle,
		serviceType: b.serviceType,
	}, nil
}

// Client is a port that sends requests and receives responses.
type Client struct {
	handle      C.iox2_client_h
	serviceType ServiceType
}

// SendCopy sends a copy of the provided data as a request and returns a PendingResponse
// to receive the corresponding responses.
func (c *Client) SendCopy(data unsafe.Pointer, sizeOfElement, numberOfElements uint64) (*PendingResponse, error) {
	if c.handle == nil {
		return nil, ErrClientClosed
	}

	var pendingResponseHandle C.iox2_pending_response_h
	result := C.iox2_client_send_copy(
		&c.handle,
		data,
		C.c_size_t(sizeOfElement),
		C.c_size_t(numberOfElements),
		nil,
		&pendingResponseHandle,
	)

	if result != C.IOX2_OK {
		return nil, RequestSendError(result)
	}

	return &PendingResponse{
		handle: pendingResponseHandle,
	}, nil
}

// SendCopyAs is a generic helper to send a copy of typed data.
func SendCopyAs[T any](c *Client, data *T) (*PendingResponse, error) {
	var zero T
	size := unsafe.Sizeof(zero)
	return c.SendCopy(unsafe.Pointer(data), uint64(size), 1)
}

// LoanSliceUninit loans memory from the client's data segment for zero-copy requests.
func (c *Client) LoanSliceUninit(numberOfElements uint64) (*RequestMut, error) {
	if c.handle == nil {
		return nil, ErrClientClosed
	}

	var requestHandle C.iox2_request_mut_h
	result := C.iox2_client_loan_slice_uninit(&c.handle, nil, &requestHandle, C.c_size_t(numberOfElements))

	if result != C.IOX2_OK {
		return nil, LoanError(result)
	}

	return &RequestMut{
		handle: requestHandle,
	}, nil
}

// Close releases the client resources.
// Implements io.Closer interface.
func (c *Client) Close() error {
	if c.handle != nil {
		C.iox2_client_drop(c.handle)
		c.handle = nil
	}
	return nil
}

// ServerBuilder is used to configure and create a Server.
type ServerBuilder struct {
	handle      C.iox2_port_factory_server_builder_h
	serviceType ServiceType
}

// InitialMaxSliceLen sets the initial maximum slice length for loan operations.
func (b *ServerBuilder) InitialMaxSliceLen(len uint64) *ServerBuilder {
	if b.handle != nil {
		C.iox2_port_factory_server_builder_set_initial_max_slice_len(&b.handle, C.c_size_t(len))
	}
	return b
}

// AllocationStrategy sets the allocation strategy for the server.
func (b *ServerBuilder) AllocationStrategy(strategy AllocationStrategy) *ServerBuilder {
	if b.handle != nil {
		C.iox2_port_factory_server_builder_set_allocation_strategy(&b.handle, uint32(strategy))
	}
	return b
}

// Create creates a new Server.
func (b *ServerBuilder) Create() (*Server, error) {
	if b.handle == nil {
		return nil, ErrBuilderConsumed
	}

	var serverHandle C.iox2_server_h
	result := C.iox2_port_factory_server_builder_create(b.handle, nil, &serverHandle)

	// Builder handle is consumed
	b.handle = nil

	if result != C.IOX2_OK {
		return nil, ServerCreateError(result)
	}

	return &Server{
		handle:      serverHandle,
		serviceType: b.serviceType,
	}, nil
}

// Server is a port that receives requests and sends responses.
// Server is safe for concurrent use; the handle is protected by an RWMutex
// to ensure Close() waits for any in-flight FFI calls to complete.
type Server struct {
	handle      C.iox2_server_h
	serviceType ServiceType
	mu          sync.RWMutex // protects handle during FFI calls
}

// HasRequests returns true if there are pending requests to be received.
func (s *Server) HasRequests() (bool, error) {
	if s.handle == nil {
		return false, ErrServerClosed
	}

	var result C.bool
	errCode := C.iox2_server_has_requests(&s.handle, &result)

	if errCode != C.IOX2_OK {
		return false, ConnectionError(errCode)
	}

	return bool(result), nil
}

// Receive receives the next request from the server queue.
// Returns ErrNoData if no request is available.
func (s *Server) Receive() (*ActiveRequest, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if s.handle == nil {
		return nil, ErrServerClosed
	}

	var requestHandle C.iox2_active_request_h
	result := C.iox2_server_receive(&s.handle, nil, &requestHandle)

	if result != C.IOX2_OK {
		return nil, ReceiveError(result)
	}

	if requestHandle == nil {
		return nil, ErrNoData
	}

	return &ActiveRequest{
		handle: requestHandle,
	}, nil
}

// ReceiveWithContext waits for a request with context cancellation support.
// This is the idiomatic Go way to receive with cancellation support.
// The pollInterval parameter controls how often the context is checked (default 10ms if 0).
func (s *Server) ReceiveWithContext(ctx context.Context, pollInterval time.Duration) (*ActiveRequest, error) {
	if s.handle == nil {
		return nil, ErrServerClosed
	}

	if pollInterval == 0 {
		pollInterval = 10 * time.Millisecond
	}

	ticker := time.NewTicker(pollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
			request, err := s.Receive()
			if errors.Is(err, ErrNoData) {
				// No request yet, wait for next poll interval
				select {
				case <-ctx.Done():
					return nil, ctx.Err()
				case <-ticker.C:
					continue
				}
			}
			if err != nil {
				return nil, err
			}
			return request, nil
		}
	}
}

// ReceiveChannel returns a channel that yields requests as they arrive.
// The channel is closed when the context is cancelled or an error occurs.
// This provides idiomatic Go channel-based integration for select statements.
func (s *Server) ReceiveChannel(ctx context.Context) <-chan *ActiveRequest {
	ch := make(chan *ActiveRequest)
	go func() {
		defer close(ch)
		for {
			request, err := s.ReceiveWithContext(ctx, 10*time.Millisecond)
			if err != nil {
				return // Context cancelled or error
			}
			select {
			case <-ctx.Done():
				return
			case ch <- request:
			}
		}
	}()
	return ch
}

// InitialMaxSliceLen returns the initial max slice length.
func (s *Server) InitialMaxSliceLen() uint64 {
	if s.handle == nil {
		return 0
	}
	return uint64(C.iox2_server_initial_max_slice_len(&s.handle))
}

// Close releases the server resources.
// Close waits for any in-flight FFI calls (e.g., from ReceiveChannel goroutines) to complete.
// Implements io.Closer interface.
func (s *Server) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.handle != nil {
		C.iox2_server_drop(s.handle)
		s.handle = nil
	}
	return nil
}

// RequestMut represents a loaned request that can be written to and sent.
type RequestMut struct {
	handle C.iox2_request_mut_h
}

// Payload returns a pointer to the request payload data.
func (r *RequestMut) Payload() unsafe.Pointer {
	if r.handle == nil {
		return nil
	}
	var ptr unsafe.Pointer
	var count C.c_size_t
	C.iox2_request_mut_payload_mut(&r.handle, &ptr, &count)
	return ptr
}

// PayloadAs returns the payload cast to the specified type.
func RequestMutPayloadAs[T any](r *RequestMut) *T {
	return (*T)(r.Payload())
}

// Send sends the request and returns a PendingResponse to receive responses.
func (r *RequestMut) Send() (*PendingResponse, error) {
	if r.handle == nil {
		return nil, ErrNilHandle
	}

	var pendingResponseHandle C.iox2_pending_response_h
	result := C.iox2_request_mut_send(r.handle, nil, &pendingResponseHandle)

	// Handle is consumed
	r.handle = nil

	if result != C.IOX2_OK {
		return nil, RequestSendError(result)
	}

	return &PendingResponse{
		handle: pendingResponseHandle,
	}, nil
}

// Close releases the request without sending.
// Implements io.Closer interface.
func (r *RequestMut) Close() error {
	if r.handle != nil {
		C.iox2_request_mut_drop(r.handle)
		r.handle = nil
	}
	return nil
}

// ActiveRequest represents a received request that can be responded to.
type ActiveRequest struct {
	handle C.iox2_active_request_h
}

// Payload returns a pointer to the request payload data.
func (r *ActiveRequest) Payload() unsafe.Pointer {
	if r.handle == nil {
		return nil
	}
	var ptr unsafe.Pointer
	var count C.c_size_t
	C.iox2_active_request_payload(&r.handle, &ptr, &count)
	return ptr
}

// PayloadAs returns the payload cast to the specified type.
func ActiveRequestPayloadAs[T any](r *ActiveRequest) *T {
	return (*T)(r.Payload())
}

// SendCopy sends a copy of the provided data as a response.
func (r *ActiveRequest) SendCopy(data unsafe.Pointer, sizeOfElement, numberOfElements uint64) error {
	if r.handle == nil {
		return ErrNilHandle
	}

	result := C.iox2_active_request_send_copy(
		&r.handle,
		data,
		C.c_size_t(sizeOfElement),
		C.c_size_t(numberOfElements),
	)

	if result != C.IOX2_OK {
		return ResponseSendError(result)
	}

	return nil
}

// SendCopyAs is a generic helper to send a copy of typed data as a response.
func ActiveRequestSendCopyAs[T any](r *ActiveRequest, data *T) error {
	var zero T
	size := unsafe.Sizeof(zero)
	return r.SendCopy(unsafe.Pointer(data), uint64(size), 1)
}

// LoanSliceUninit loans memory for a zero-copy response.
func (r *ActiveRequest) LoanSliceUninit(numberOfElements uint64) (*ResponseMut, error) {
	if r.handle == nil {
		return nil, ErrNilHandle
	}

	var responseHandle C.iox2_response_mut_h
	result := C.iox2_active_request_loan_slice_uninit(&r.handle, nil, &responseHandle, C.c_size_t(numberOfElements))

	if result != C.IOX2_OK {
		return nil, LoanError(result)
	}

	return &ResponseMut{
		handle: responseHandle,
	}, nil
}

// Close releases the active request resources.
// Implements io.Closer interface.
func (r *ActiveRequest) Close() error {
	if r.handle != nil {
		C.iox2_active_request_drop(r.handle)
		r.handle = nil
	}
	return nil
}

// ResponseMut represents a loaned response that can be written to and sent.
type ResponseMut struct {
	handle C.iox2_response_mut_h
}

// Payload returns a pointer to the response payload data.
func (r *ResponseMut) Payload() unsafe.Pointer {
	if r.handle == nil {
		return nil
	}
	var ptr unsafe.Pointer
	var count C.c_size_t
	C.iox2_response_mut_payload_mut(&r.handle, &ptr, &count)
	return ptr
}

// PayloadAs returns the payload cast to the specified type.
func ResponseMutPayloadAs[T any](r *ResponseMut) *T {
	return (*T)(r.Payload())
}

// Send sends the response.
func (r *ResponseMut) Send() error {
	if r.handle == nil {
		return ErrNilHandle
	}

	result := C.iox2_response_mut_send(r.handle)

	// Handle is consumed
	r.handle = nil

	if result != C.IOX2_OK {
		return ResponseSendError(result)
	}

	return nil
}

// Close releases the response without sending.
// Implements io.Closer interface.
func (r *ResponseMut) Close() error {
	if r.handle != nil {
		C.iox2_response_mut_drop(r.handle)
		r.handle = nil
	}
	return nil
}

// PendingResponse represents a sent request that is awaiting responses.
// PendingResponse is safe for concurrent use; the handle is protected by an RWMutex
// to ensure Close() waits for any in-flight FFI calls to complete.
type PendingResponse struct {
	handle C.iox2_pending_response_h
	mu     sync.RWMutex // protects handle during FFI calls
}

// Receive receives the next response for this request.
// Returns ErrNoData if no response is available yet.
func (p *PendingResponse) Receive() (*Response, error) {
	p.mu.RLock()
	defer p.mu.RUnlock()

	if p.handle == nil {
		return nil, ErrNilHandle
	}

	var responseHandle C.iox2_response_h
	result := C.iox2_pending_response_receive(&p.handle, nil, &responseHandle)

	if result != C.IOX2_OK {
		return nil, ReceiveError(result)
	}

	if responseHandle == nil {
		return nil, ErrNoData
	}

	return &Response{
		handle: responseHandle,
	}, nil
}

// ReceiveWithContext waits for a response with context cancellation support.
// This is the idiomatic Go way to receive with cancellation support.
// The pollInterval parameter controls how often the context is checked (default 10ms if 0).
func (p *PendingResponse) ReceiveWithContext(ctx context.Context, pollInterval time.Duration) (*Response, error) {
	if p.handle == nil {
		return nil, ErrNilHandle
	}

	if pollInterval == 0 {
		pollInterval = 10 * time.Millisecond
	}

	ticker := time.NewTicker(pollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
			response, err := p.Receive()
			if errors.Is(err, ErrNoData) {
				// No response yet, wait for next poll interval
				select {
				case <-ctx.Done():
					return nil, ctx.Err()
				case <-ticker.C:
					continue
				}
			}
			if err != nil {
				return nil, err
			}
			return response, nil
		}
	}
}

// ReceiveChannel returns a channel that yields responses as they arrive.
// The channel is closed when the context is cancelled or an error occurs.
// This provides idiomatic Go channel-based integration for select statements.
func (p *PendingResponse) ReceiveChannel(ctx context.Context) <-chan *Response {
	ch := make(chan *Response)
	go func() {
		defer close(ch)
		for {
			response, err := p.ReceiveWithContext(ctx, 10*time.Millisecond)
			if err != nil {
				return // Context cancelled or error
			}
			select {
			case <-ctx.Done():
				response.Close()
				return
			case ch <- response:
			}
		}
	}()
	return ch
}

// Close releases the pending response resources.
// Close waits for any in-flight FFI calls (e.g., from ReceiveChannel goroutines) to complete.
// Implements io.Closer interface.
func (p *PendingResponse) Close() error {
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.handle != nil {
		C.iox2_pending_response_drop(p.handle)
		p.handle = nil
	}
	return nil
}

// Response represents a received response.
type Response struct {
	handle C.iox2_response_h
}

// Payload returns a pointer to the response payload data.
func (r *Response) Payload() unsafe.Pointer {
	if r.handle == nil {
		return nil
	}
	var ptr unsafe.Pointer
	var count C.c_size_t
	C.iox2_response_payload(&r.handle, &ptr, &count)
	return ptr
}

// PayloadAs returns the payload cast to the specified type.
func ResponsePayloadAs[T any](r *Response) *T {
	return (*T)(r.Payload())
}

// Close releases the response resources.
// Implements io.Closer interface.
func (r *Response) Close() error {
	if r.handle != nil {
		C.iox2_response_drop(r.handle)
		r.handle = nil
	}
	return nil
}
