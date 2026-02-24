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
	"testing"
	"unsafe"
)

// BenchmarkPayload is a typical payload for benchmarking.
type BenchmarkPayload struct {
	Timestamp int64
	Counter   int32
	Value     float64
	Data      [64]byte
}

// BenchmarkPublishSend measures the time to loan and send a sample.
func BenchmarkPublishSend(b *testing.B) {
	node, err := NewNodeBuilder().Create(ServiceTypeLocal)
	if err != nil {
		b.Fatalf("failed to create node: %v", err)
	}
	defer node.Close()

	serviceName, err := NewServiceName("benchmark/pubsub")
	if err != nil {
		b.Fatalf("failed to create service name: %v", err)
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		PublishSubscribe().
		PayloadType("BenchmarkPayload",
			uint64(unsafe.Sizeof(BenchmarkPayload{})),
			uint64(unsafe.Alignof(BenchmarkPayload{}))).
		OpenOrCreate()
	if err != nil {
		b.Fatalf("failed to create service: %v", err)
	}
	defer service.Close()

	publisher, err := service.PublisherBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create publisher: %v", err)
	}
	defer publisher.Close()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		sample, err := publisher.LoanUninit()
		if err != nil {
			b.Fatalf("failed to loan sample: %v", err)
		}

		payload := PayloadMutAs[BenchmarkPayload](sample)
		payload.Counter = int32(i)
		payload.Timestamp = int64(i)

		if err := sample.Send(); err != nil {
			b.Fatalf("failed to send: %v", err)
		}
	}
}

// BenchmarkPublishReceive measures roundtrip time for publish and receive.
func BenchmarkPublishReceive(b *testing.B) {
	node, err := NewNodeBuilder().Create(ServiceTypeLocal)
	if err != nil {
		b.Fatalf("failed to create node: %v", err)
	}
	defer node.Close()

	serviceName, err := NewServiceName("benchmark/pubsub/roundtrip")
	if err != nil {
		b.Fatalf("failed to create service name: %v", err)
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		PublishSubscribe().
		PayloadType("BenchmarkPayload",
			uint64(unsafe.Sizeof(BenchmarkPayload{})),
			uint64(unsafe.Alignof(BenchmarkPayload{}))).
		OpenOrCreate()
	if err != nil {
		b.Fatalf("failed to create service: %v", err)
	}
	defer service.Close()

	publisher, err := service.PublisherBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create publisher: %v", err)
	}
	defer publisher.Close()

	subscriber, err := service.SubscriberBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create subscriber: %v", err)
	}
	defer subscriber.Close()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		// Send
		sample, err := publisher.LoanUninit()
		if err != nil {
			b.Fatalf("failed to loan sample: %v", err)
		}

		payload := PayloadMutAs[BenchmarkPayload](sample)
		payload.Counter = int32(i)

		if err := sample.Send(); err != nil {
			b.Fatalf("failed to send: %v", err)
		}

		// Receive
		received, err := subscriber.Receive()
		if err != nil {
			b.Fatalf("failed to receive: %v", err)
		}
		received.Close()
	}
}

// BenchmarkEventNotify measures the time to send an event notification.
func BenchmarkEventNotify(b *testing.B) {
	node, err := NewNodeBuilder().Create(ServiceTypeLocal)
	if err != nil {
		b.Fatalf("failed to create node: %v", err)
	}
	defer node.Close()

	serviceName, err := NewServiceName("benchmark/event")
	if err != nil {
		b.Fatalf("failed to create service name: %v", err)
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		Event().
		OpenOrCreate()
	if err != nil {
		b.Fatalf("failed to create service: %v", err)
	}
	defer service.Close()

	notifier, err := service.NotifierBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create notifier: %v", err)
	}
	defer notifier.Close()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		if _, err := notifier.Notify(); err != nil {
			b.Fatalf("failed to notify: %v", err)
		}
	}
}

// BenchmarkEventRoundtrip measures the time for event notify and wait.
func BenchmarkEventRoundtrip(b *testing.B) {
	node, err := NewNodeBuilder().Create(ServiceTypeLocal)
	if err != nil {
		b.Fatalf("failed to create node: %v", err)
	}
	defer node.Close()

	serviceName, err := NewServiceName("benchmark/event/roundtrip")
	if err != nil {
		b.Fatalf("failed to create service name: %v", err)
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		Event().
		OpenOrCreate()
	if err != nil {
		b.Fatalf("failed to create service: %v", err)
	}
	defer service.Close()

	notifier, err := service.NotifierBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create notifier: %v", err)
	}
	defer notifier.Close()

	listener, err := service.ListenerBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create listener: %v", err)
	}
	defer listener.Close()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		if _, err := notifier.Notify(); err != nil {
			b.Fatalf("failed to notify: %v", err)
		}

		if _, err := listener.TryWaitOne(); err != nil {
			b.Fatalf("failed to wait: %v", err)
		}
	}
}

// BenchmarkNodeCreate measures the time to create and destroy a node.
func BenchmarkNodeCreate(b *testing.B) {
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		node, err := NewNodeBuilder().Create(ServiceTypeLocal)
		if err != nil {
			b.Fatalf("failed to create node: %v", err)
		}
		node.Close()
	}
}

// BenchmarkServiceNameCreate measures the time to create a service name.
func BenchmarkServiceNameCreate(b *testing.B) {
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		name, err := NewServiceName("benchmark/service/name")
		if err != nil {
			b.Fatalf("failed to create service name: %v", err)
		}
		name.Close()
	}
}

// BenchmarkReceiveEmpty measures the time to check for messages when none exist.
func BenchmarkReceiveEmpty(b *testing.B) {
	node, err := NewNodeBuilder().Create(ServiceTypeLocal)
	if err != nil {
		b.Fatalf("failed to create node: %v", err)
	}
	defer node.Close()

	serviceName, err := NewServiceName("benchmark/receive/empty")
	if err != nil {
		b.Fatalf("failed to create service name: %v", err)
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		PublishSubscribe().
		PayloadType("int32", 4, 4).
		OpenOrCreate()
	if err != nil {
		b.Fatalf("failed to create service: %v", err)
	}
	defer service.Close()

	subscriber, err := service.SubscriberBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create subscriber: %v", err)
	}
	defer subscriber.Close()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		// This should return ErrNoData quickly
		_, _ = subscriber.Receive()
	}
}

// BenchmarkLoanOnly measures the overhead of just loaning memory (without sending).
func BenchmarkLoanOnly(b *testing.B) {
	node, err := NewNodeBuilder().Create(ServiceTypeLocal)
	if err != nil {
		b.Fatalf("failed to create node: %v", err)
	}
	defer node.Close()

	serviceName, err := NewServiceName("benchmark/loan")
	if err != nil {
		b.Fatalf("failed to create service name: %v", err)
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		PublishSubscribe().
		PayloadType("BenchmarkPayload",
			uint64(unsafe.Sizeof(BenchmarkPayload{})),
			uint64(unsafe.Alignof(BenchmarkPayload{}))).
		MaxPublishers(1).
		HistorySize(0).
		OpenOrCreate()
	if err != nil {
		b.Fatalf("failed to create service: %v", err)
	}
	defer service.Close()

	publisher, err := service.PublisherBuilder().Create()
	if err != nil {
		b.Fatalf("failed to create publisher: %v", err)
	}
	defer publisher.Close()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		sample, err := publisher.LoanUninit()
		if err != nil {
			b.Fatalf("failed to loan sample: %v", err)
		}
		// Drop without sending
		sample.Close()
	}
}
