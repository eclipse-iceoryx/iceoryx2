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

import (
	"context"
	"io"
)

// Ensure types implement io.Closer at compile time.
var (
	_ io.Closer = (*Node)(nil)
	_ io.Closer = (*ServiceName)(nil)
	_ io.Closer = (*NodeName)(nil)
	_ io.Closer = (*NodeId)(nil)
	_ io.Closer = (*Publisher)(nil)
	_ io.Closer = (*Subscriber)(nil)
	_ io.Closer = (*Sample)(nil)
	_ io.Closer = (*SampleMut)(nil)
	_ io.Closer = (*PortFactoryPubSub)(nil)
	_ io.Closer = (*PortFactoryEvent)(nil)
	_ io.Closer = (*PortFactoryRequestResponse)(nil)
	_ io.Closer = (*Notifier)(nil)
	_ io.Closer = (*Listener)(nil)
	_ io.Closer = (*Client)(nil)
	_ io.Closer = (*Server)(nil)
	_ io.Closer = (*RequestMut)(nil)
	_ io.Closer = (*ActiveRequest)(nil)
	_ io.Closer = (*ResponseMut)(nil)
	_ io.Closer = (*Response)(nil)
	_ io.Closer = (*PendingResponse)(nil)
	_ io.Closer = (*WaitSet)(nil)
	_ io.Closer = (*WaitSetGuard)(nil)
)

// PublisherPort defines the interface for publishing messages.
type PublisherPort interface {
	io.Closer

	// LoanUninit loans an uninitialized sample for zero-copy writing.
	LoanUninit() (*SampleMut, error)

	// LoanSliceUninit loans an uninitialized sample with specified capacity.
	LoanSliceUninit(len uint64) (*SampleMut, error)

	// Send sends the given data (with copy).
	Send(data []byte) error
}

// SubscriberPort defines the interface for receiving messages.
type SubscriberPort interface {
	io.Closer

	// Receive receives a sample from the subscriber's buffer.
	// Returns nil if no sample is available.
	Receive() (*Sample, error)
}

// NotifierPort defines the interface for sending event notifications.
type NotifierPort interface {
	io.Closer

	// Notify sends a notification with the default event ID.
	Notify() (uint64, error)

	// NotifyWithEventId sends a notification with a specific event ID.
	NotifyWithEventId(eventId uint64) (uint64, error)
}

// ListenerPort defines the interface for receiving event notifications.
type ListenerPort interface {
	io.Closer

	// TryWaitOne tries to receive a single event without blocking.
	TryWaitOne() (*EventId, error)

	// WaitOne waits for a single event with context support.
	WaitOne(ctx context.Context) (*EventId, error)

	// TryWaitAll receives all pending events.
	TryWaitAll() ([]EventId, error)
}

// ClientPort defines the interface for request-response clients.
type ClientPort interface {
	io.Closer

	// LoanSliceUninit loans memory for zero-copy requests.
	LoanSliceUninit(numberOfElements uint64) (*RequestMut, error)
}

// ServerPort defines the interface for request-response servers.
type ServerPort interface {
	io.Closer

	// HasRequests returns true if there are pending requests.
	HasRequests() (bool, error)

	// Receive receives the next request.
	Receive() (*ActiveRequest, error)
}
