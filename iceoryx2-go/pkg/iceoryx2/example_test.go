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

package iceoryx2_test

import (
	"context"
	"fmt"
	"time"
	"unsafe"

	"github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

// ExamplePayload is a simple payload structure for examples.
type ExamplePayload struct {
	Counter int32
	Value   float64
}

func Example_createNode() {
	// Create a node with the IPC service type for inter-process communication.
	node, err := iceoryx2.NewNodeBuilder().
		Name("example-node").
		Create(iceoryx2.ServiceTypeIpc)
	if err != nil {
		fmt.Printf("Failed to create node: %v\n", err)
		return
	}
	defer node.Close()

	fmt.Printf("Node created with ID: %v\n", node.ID())
	// Output is dynamic due to unique node ID, so we just verify it runs
}

func Example_publishSubscribe() {
	// Create a node
	node, err := iceoryx2.NewNodeBuilder().Create(iceoryx2.ServiceTypeLocal)
	if err != nil {
		fmt.Printf("Failed to create node: %v\n", err)
		return
	}
	defer node.Close()

	// Create a service name
	serviceName, err := iceoryx2.NewServiceName("example/pubsub")
	if err != nil {
		fmt.Printf("Failed to create service name: %v\n", err)
		return
	}
	defer serviceName.Close()

	// Create a publish-subscribe service
	service, err := node.ServiceBuilder(serviceName).
		PublishSubscribe().
		PayloadType("ExamplePayload",
			uint64(unsafe.Sizeof(ExamplePayload{})),
			uint64(unsafe.Alignof(ExamplePayload{}))).
		OpenOrCreate()
	if err != nil {
		fmt.Printf("Failed to create service: %v\n", err)
		return
	}
	defer service.Close()

	// Create a publisher
	publisher, err := service.PublisherBuilder().Create()
	if err != nil {
		fmt.Printf("Failed to create publisher: %v\n", err)
		return
	}
	defer publisher.Close()

	// Create a subscriber
	subscriber, err := service.SubscriberBuilder().Create()
	if err != nil {
		fmt.Printf("Failed to create subscriber: %v\n", err)
		return
	}
	defer subscriber.Close()

	// Publish data using zero-copy
	sample, err := publisher.LoanUninit()
	if err != nil {
		fmt.Printf("Failed to loan sample: %v\n", err)
		return
	}

	// Write payload directly to shared memory
	payload := iceoryx2.PayloadMutAs[ExamplePayload](sample)
	payload.Counter = 42
	payload.Value = 3.14

	if err := sample.Send(); err != nil {
		fmt.Printf("Failed to send: %v\n", err)
		return
	}

	// Receive the data
	received, err := subscriber.Receive()
	if err != nil {
		fmt.Printf("Failed to receive: %v\n", err)
		return
	}
	defer received.Close()

	data := iceoryx2.PayloadAs[ExamplePayload](received)
	fmt.Printf("Received: Counter=%d, Value=%.2f\n", data.Counter, data.Value)
	// Output: Received: Counter=42, Value=3.14
}

func Example_eventNotification() {
	// Create a node
	node, err := iceoryx2.NewNodeBuilder().Create(iceoryx2.ServiceTypeLocal)
	if err != nil {
		fmt.Printf("Failed to create node: %v\n", err)
		return
	}
	defer node.Close()

	// Create a service name
	serviceName, err := iceoryx2.NewServiceName("example/event")
	if err != nil {
		fmt.Printf("Failed to create service name: %v\n", err)
		return
	}
	defer serviceName.Close()

	// Create an event service
	service, err := node.ServiceBuilder(serviceName).
		Event().
		OpenOrCreate()
	if err != nil {
		fmt.Printf("Failed to create event service: %v\n", err)
		return
	}
	defer service.Close()

	// Create a notifier
	notifier, err := service.NotifierBuilder().Create()
	if err != nil {
		fmt.Printf("Failed to create notifier: %v\n", err)
		return
	}
	defer notifier.Close()

	// Create a listener
	listener, err := service.ListenerBuilder().Create()
	if err != nil {
		fmt.Printf("Failed to create listener: %v\n", err)
		return
	}
	defer listener.Close()

	// Send an event with a custom ID
	eventId := uint64(123)
	if _, err := notifier.NotifyWithEventId(eventId); err != nil {
		fmt.Printf("Failed to notify: %v\n", err)
		return
	}

	// Receive the event (non-blocking)
	receivedId, err := listener.TryWaitOne()
	if err != nil {
		fmt.Printf("Failed to wait: %v\n", err)
		return
	}

	fmt.Printf("Received event ID: %d\n", *receivedId)
	// Output: Received event ID: 123
}

func ExampleSubscriber_ReceiveWithContext() {
	// Create a node
	node, _ := iceoryx2.NewNodeBuilder().Create(iceoryx2.ServiceTypeLocal)
	defer node.Close()

	serviceName, _ := iceoryx2.NewServiceName("example/context")
	defer serviceName.Close()

	service, _ := node.ServiceBuilder(serviceName).
		PublishSubscribe().
		PayloadType("int32", 4, 4).
		OpenOrCreate()
	defer service.Close()

	subscriber, _ := service.SubscriberBuilder().Create()
	defer subscriber.Close()

	// Use context for cancellation
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	// This will return context.DeadlineExceeded after 100ms since no data is sent
	_, err := subscriber.ReceiveWithContext(ctx, 10*time.Millisecond)
	if err == context.DeadlineExceeded {
		fmt.Println("Receive timed out as expected")
	}
	// Output: Receive timed out as expected
}

func ExampleNewServiceName() {
	// Service names follow a path-like format
	serviceName, err := iceoryx2.NewServiceName("my/service/name")
	if err != nil {
		fmt.Printf("Invalid service name: %v\n", err)
		return
	}
	defer serviceName.Close()

	fmt.Println("Service name created successfully")
	// Output: Service name created successfully
}

func ExampleNewServiceName_invalid() {
	// Empty names are invalid
	_, err := iceoryx2.NewServiceName("")
	if err != nil {
		fmt.Println("Empty service name rejected")
	}
	// Output: Empty service name rejected
}
