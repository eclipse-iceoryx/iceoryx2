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

// StaticConfigPubSub contains the static configuration of a publish-subscribe service.
// These values are set at service creation and cannot be changed.
type StaticConfigPubSub struct {
	// MaxSubscribers is the maximum number of subscribers the service supports.
	MaxSubscribers uint64
	// MaxPublishers is the maximum number of publishers the service supports.
	MaxPublishers uint64
	// MaxNodes is the maximum number of nodes that can use this service.
	MaxNodes uint64
	// HistorySize is the number of samples stored for late-joining subscribers.
	HistorySize uint64
	// SubscriberMaxBufferSize is the maximum buffer size for subscribers.
	SubscriberMaxBufferSize uint64
	// SubscriberMaxBorrowedSamples is the maximum number of samples a subscriber can borrow at once.
	SubscriberMaxBorrowedSamples uint64
	// EnableSafeOverflow indicates if safe overflow behavior is enabled.
	EnableSafeOverflow bool
	// MessageTypeDetails contains information about the message type.
	MessageTypeDetails MessageTypeDetails
}

// MessageTypeDetails contains information about the payload and header types.
type MessageTypeDetails struct {
	// PayloadTypeName is the name of the payload type.
	PayloadTypeName string
	// PayloadSize is the size of the payload in bytes.
	PayloadSize uint64
	// PayloadAlignment is the alignment requirement of the payload.
	PayloadAlignment uint64
	// UserHeaderTypeName is the name of the user header type.
	UserHeaderTypeName string
	// UserHeaderSize is the size of the user header in bytes.
	UserHeaderSize uint64
	// UserHeaderAlignment is the alignment requirement of the user header.
	UserHeaderAlignment uint64
}

// StaticConfigEvent contains the static configuration of an event service.
type StaticConfigEvent struct {
	// MaxListeners is the maximum number of listeners the service supports.
	MaxListeners uint64
	// MaxNotifiers is the maximum number of notifiers the service supports.
	MaxNotifiers uint64
	// MaxNodes is the maximum number of nodes that can use this service.
	MaxNodes uint64
	// EventIdMaxValue is the maximum value for event IDs.
	EventIdMaxValue uint64
}

// PublisherDetails contains information about a connected publisher.
type PublisherDetails struct {
	// PublisherID is the unique ID of the publisher.
	PublisherID UniquePublisherId
	// NodeID is the ID of the node owning this publisher.
	NodeID NodeId
}

// SubscriberDetails contains information about a connected subscriber.
type SubscriberDetails struct {
	// SubscriberID is the unique ID of the subscriber.
	SubscriberID UniqueSubscriberId
	// NodeID is the ID of the node owning this subscriber.
	NodeID NodeId
}

// ListenerDetails contains information about a connected listener.
type ListenerDetails struct {
	// ListenerID is the unique ID of the listener.
	ListenerID UniqueListenerId
	// NodeID is the ID of the node owning this listener.
	NodeID NodeId
}

// NotifierDetails contains information about a connected notifier.
type NotifierDetails struct {
	// NotifierID is the unique ID of the notifier.
	NotifierID UniqueNotifierId
	// NodeID is the ID of the node owning this notifier.
	NodeID NodeId
}
