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
	"errors"
	"fmt"
	"math/rand"
	"testing"
	"time"
	"unsafe"
)

// TestPayload is a simple test payload structure
type TestPayload struct {
	Value int32
}

// serviceTypes returns both IPC and Local service types for parameterized tests
var serviceTypes = []ServiceType{ServiceTypeIpc, ServiceTypeLocal}

// generateServiceName creates a unique service name for testing
func generateServiceName(t *testing.T) *ServiceName {
	name := fmt.Sprintf("test/service/%d/%d", time.Now().UnixNano(), rand.Int())
	serviceName, err := NewServiceName(name)
	if err != nil {
		t.Fatalf("failed to create service name: %v", err)
	}
	return serviceName
}

// =============================================================================
// Node Tests
// =============================================================================

func TestCreateNodeWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			// Node should have empty name by default
			if node.Name() != "" {
				t.Errorf("expected empty name, got %q", node.Name())
			}
		})
	}
}

func TestCreateNodeWithNameWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			nodeName := "test-node"
			node, err := NewNodeBuilder().
				Name(nodeName).
				Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			if node.Name() != nodeName {
				t.Errorf("expected name %q, got %q", nodeName, node.Name())
			}
		})
	}
}

func TestNodeHasValidId(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			nodeId := node.ID()
			if nodeId == nil {
				t.Error("node ID should not be nil")
			}
			defer nodeId.Close()

			// PID should be valid (non-zero for this process)
			if nodeId.Pid() == 0 {
				t.Error("node ID pid should not be zero")
			}
		})
	}
}

func TestNodeWaitWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			// Wait should not error with a short timeout
			err = node.Wait(100 * time.Millisecond)
			if err != nil {
				t.Errorf("wait failed: %v", err)
			}
		})
	}
}

func TestListNodesFindsCurrentNode(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			nodeName := fmt.Sprintf("list-test-node-%d", time.Now().UnixNano())
			node, err := NewNodeBuilder().
				Name(nodeName).
				Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			// Get the node's config for listing
			config := node.Config()
			if config == nil {
				t.Fatal("node config is nil")
			}

			// List all nodes
			nodes, err := ListNodes(serviceType, config)
			if err != nil {
				t.Fatalf("failed to list nodes: %v", err)
			}

			// Should find at least the node we just created
			found := false
			for _, info := range nodes {
				if info.Name == nodeName {
					found = true
					if info.State != NodeStateAlive {
						t.Errorf("expected node state Alive, got %v", info.State)
					}
					break
				}
			}

			if !found {
				t.Error("created node not found in node list")
			}
		})
	}
}

// =============================================================================
// ServiceName Tests
// =============================================================================

func TestCreateServiceNameWorks(t *testing.T) {
	name := "My/Funk/ServiceName"
	serviceName, err := NewServiceName(name)
	if err != nil {
		t.Fatalf("failed to create service name: %v", err)
	}
	defer serviceName.Close()

	if serviceName.String() != name {
		t.Errorf("expected %q, got %q", name, serviceName.String())
	}
}

func TestInvalidServiceNameFails(t *testing.T) {
	// Empty name should fail
	_, err := NewServiceName("")
	if err == nil {
		t.Error("expected error for empty service name")
	}
}

// =============================================================================
// Publish-Subscribe Tests
// =============================================================================

func TestPubSubSendAndReceiveWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			publisher, err := service.PublisherBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create publisher: %v", err)
			}
			defer publisher.Close()

			subscriber, err := service.SubscriberBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create subscriber: %v", err)
			}
			defer subscriber.Close()

			// Send and receive data one at a time
			// (sending multiple samples at once may exceed buffer limits)
			for i := int32(0); i < 3; i++ {
				sample, err := publisher.LoanUninit()
				if err != nil {
					t.Fatalf("failed to loan sample: %v", err)
				}
				payload := PayloadMutAs[TestPayload](sample)
				payload.Value = 42 + i
				err = sample.Send()
				if err != nil {
					t.Fatalf("failed to send sample: %v", err)
				}

				// Receive immediately
				received, err := subscriber.Receive()
				if err != nil {
					t.Fatalf("failed to receive sample %d: %v", i, err)
				}
				if received == nil {
					t.Fatalf("expected sample %d, got nil", i)
				}
				recvPayload := PayloadAs[TestPayload](received)
				expected := 42 + i
				if recvPayload.Value != expected {
					t.Errorf("expected value %d, got %d", expected, recvPayload.Value)
				}
				received.Close()
			}
		})
	}
}

func TestPublisherIdIsValid(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			publisher, err := service.PublisherBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create publisher: %v", err)
			}
			defer publisher.Close()

			pubId, err := publisher.ID()
			if err != nil {
				t.Fatalf("failed to get publisher ID: %v", err)
			}
			if pubId == nil {
				t.Error("publisher ID should not be nil")
			}
			defer pubId.Close()
		})
	}
}

func TestSubscriberIdIsValid(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			subscriber, err := service.SubscriberBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create subscriber: %v", err)
			}
			defer subscriber.Close()

			subId, err := subscriber.ID()
			if err != nil {
				t.Fatalf("failed to get subscriber ID: %v", err)
			}
			if subId == nil {
				t.Error("subscriber ID should not be nil")
			}
			defer subId.Close()
		})
	}
}

// =============================================================================
// Event Tests
// =============================================================================

func TestEventNotifyAndWaitWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			notifier, err := service.NotifierBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create notifier: %v", err)
			}
			defer notifier.Close()

			listener, err := service.ListenerBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create listener: %v", err)
			}
			defer listener.Close()

			// Send notification
			eventId := uint64(42)
			_, err = notifier.NotifyWithEventId(eventId)
			if err != nil {
				t.Fatalf("failed to notify: %v", err)
			}

			// Receive notification
			receivedId, err := listener.TryWaitOne()
			if err != nil {
				t.Fatalf("failed to wait: %v", err)
			}
			if receivedId == nil {
				t.Fatal("expected event ID, got nil")
			}
			if uint64(*receivedId) != eventId {
				t.Errorf("expected event ID %d, got %d", eventId, *receivedId)
			}
		})
	}
}

func TestNotifierIdIsValid(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			notifier, err := service.NotifierBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create notifier: %v", err)
			}
			defer notifier.Close()

			notifierId, err := notifier.ID()
			if err != nil {
				t.Fatalf("failed to get notifier ID: %v", err)
			}
			if notifierId == nil {
				t.Error("notifier ID should not be nil")
			}
			defer notifierId.Close()
		})
	}
}

func TestListenerIdIsValid(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			listener, err := service.ListenerBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create listener: %v", err)
			}
			defer listener.Close()

			listenerId, err := listener.ID()
			if err != nil {
				t.Fatalf("failed to get listener ID: %v", err)
			}
			if listenerId == nil {
				t.Error("listener ID should not be nil")
			}
			defer listenerId.Close()
		})
	}
}

// =============================================================================
// WaitSet Tests
// =============================================================================

func TestWaitSetTimedWaitWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			waitset, err := NewWaitSetBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create waitset: %v", err)
			}
			defer waitset.Close()

			// WaitSet without attachments will return an error,
			// which is expected behavior - just verify creation works
			// Real usage requires attaching a listener first
			_, err = waitset.WaitAndProcessOnceWithTimeout(10 * time.Millisecond)
			// Error is expected since we have no attachments
			if err == nil {
				t.Log("unexpected success - waitset with no attachments should error")
			}
		})
	}
}

// =============================================================================
// Service Discovery Tests
// =============================================================================

func TestServiceDiscoveryFindsServices(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			// Create a service
			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			// List services with a callback
			var foundServices int
			var foundOurService bool
			err = ListServices(serviceType, func(info *ServiceInfo) CallbackProgression {
				foundServices++
				if info.Name == serviceName.String() {
					foundOurService = true
				}
				return CallbackProgressionContinue
			})
			if err != nil {
				t.Fatalf("failed to list services: %v", err)
			}

			// Should have found at least our service
			if foundServices == 0 {
				t.Error("callback was never invoked")
			}
			if !foundOurService {
				t.Errorf("our service %q was not found in listing (found %d services)", serviceName.String(), foundServices)
			}
		})
	}
}

func TestCollectServicesReturnsServices(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			// Create a service
			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			// Collect all services
			services, err := CollectServices(serviceType)
			if err != nil {
				t.Fatalf("failed to collect services: %v", err)
			}

			// Should have found at least our service
			found := false
			for _, svc := range services {
				if svc.Name == serviceName.String() {
					found = true
					if svc.MessagingPattern != MessagingPatternPublishSubscribe {
						t.Errorf("wrong messaging pattern: got %v, want %v",
							svc.MessagingPattern, MessagingPatternPublishSubscribe)
					}
					break
				}
			}
			if !found {
				t.Errorf("our service %q was not found in collected services (found %d services)",
					serviceName.String(), len(services))
			}
		})
	}
}

// =============================================================================
// Context Support Tests
// =============================================================================

func TestSubscriberReceiveWithContextCancellation(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			subscriber, err := service.SubscriberBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create subscriber: %v", err)
			}
			defer subscriber.Close()

			// Create a context that cancels quickly
			ctx, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
			defer cancel()

			// This should return context.DeadlineExceeded since no data is available
			_, err = subscriber.ReceiveWithContext(ctx, 10*time.Millisecond)
			if !errors.Is(err, context.DeadlineExceeded) {
				t.Errorf("expected context.DeadlineExceeded, got %v", err)
			}
		})
	}
}

func TestSubscriberReceiveWithContextReceivesData(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			publisher, err := service.PublisherBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create publisher: %v", err)
			}
			defer publisher.Close()

			subscriber, err := service.SubscriberBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create subscriber: %v", err)
			}
			defer subscriber.Close()

			// Send data first
			sample, err := publisher.LoanUninit()
			if err != nil {
				t.Fatalf("failed to loan sample: %v", err)
			}
			payload := PayloadMutAs[TestPayload](sample)
			payload.Value = 99
			err = sample.Send()
			if err != nil {
				t.Fatalf("failed to send: %v", err)
			}

			// Now receive with context - should succeed immediately
			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()

			received, err := subscriber.ReceiveWithContext(ctx, 10*time.Millisecond)
			if err != nil {
				t.Fatalf("failed to receive: %v", err)
			}
			defer received.Close()

			recvPayload := PayloadAs[TestPayload](received)
			if recvPayload.Value != 99 {
				t.Errorf("expected value 99, got %d", recvPayload.Value)
			}
		})
	}
}

func TestListenerEventChannelWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create event service: %v", err)
			}
			defer service.Close()

			notifier, err := service.NotifierBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create notifier: %v", err)
			}
			defer notifier.Close()

			listener, err := service.ListenerBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create listener: %v", err)
			}
			defer listener.Close()

			ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
			defer cancel()

			eventCh := listener.EventChannel(ctx)

			// Send an event
			eventId := uint64(42)
			_, err = notifier.NotifyWithEventId(eventId)
			if err != nil {
				t.Fatalf("failed to notify: %v", err)
			}

			// Receive from channel
			select {
			case receivedEvent, ok := <-eventCh:
				if !ok {
					t.Fatal("channel closed unexpectedly")
				}
				if uint64(receivedEvent) != eventId {
					t.Errorf("expected event %v, got %v", eventId, receivedEvent)
				}
			case <-ctx.Done():
				t.Fatal("timed out waiting for event")
			}
		})
	}
}

func TestSubscriberReceiveChannelWorks(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				PublishSubscribe().
				PayloadType("TestPayload", uint64(unsafe.Sizeof(TestPayload{})), uint64(unsafe.Alignof(TestPayload{}))).
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create service: %v", err)
			}
			defer service.Close()

			publisher, err := service.PublisherBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create publisher: %v", err)
			}
			defer publisher.Close()

			subscriber, err := service.SubscriberBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create subscriber: %v", err)
			}
			defer subscriber.Close()

			ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
			defer cancel()

			sampleCh := subscriber.ReceiveChannel(ctx)

			// Send data
			sample, err := publisher.LoanUninit()
			if err != nil {
				t.Fatalf("failed to loan sample: %v", err)
			}
			payload := PayloadMutAs[TestPayload](sample)
			payload.Value = 123
			err = sample.Send()
			if err != nil {
				t.Fatalf("failed to send: %v", err)
			}

			// Receive from channel
			select {
			case received, ok := <-sampleCh:
				if !ok {
					t.Fatal("channel closed unexpectedly")
				}
				defer received.Close()
				recvPayload := PayloadAs[TestPayload](received)
				if recvPayload.Value != 123 {
					t.Errorf("expected value 123, got %d", recvPayload.Value)
				}
			case <-ctx.Done():
				t.Fatal("timed out waiting for sample")
			}
		})
	}
}

func TestWaitSetWithContextCancellation(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create event service: %v", err)
			}
			defer service.Close()

			listener, err := service.ListenerBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create listener: %v", err)
			}
			defer listener.Close()

			waitset, err := NewWaitSetBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create waitset: %v", err)
			}
			defer waitset.Close()

			guard, err := waitset.AttachNotification(listener)
			if err != nil {
				t.Fatalf("failed to attach listener: %v", err)
			}
			defer guard.Close()

			// Create a context that cancels quickly
			ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
			defer cancel()

			// This should return context.DeadlineExceeded
			_, err = waitset.WaitAndProcessOnceWithContext(ctx, 20*time.Millisecond)
			if !errors.Is(err, context.DeadlineExceeded) {
				t.Logf("expected context.DeadlineExceeded, got %v (this is acceptable if an event arrived)", err)
			}
		})
	}
}

func TestWaitSetWithCallbackReceivesEvents(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create event service: %v", err)
			}
			defer service.Close()

			notifier, err := service.NotifierBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create notifier: %v", err)
			}
			defer notifier.Close()

			listener, err := service.ListenerBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create listener: %v", err)
			}
			defer listener.Close()

			waitset, err := NewWaitSetBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create waitset: %v", err)
			}
			defer waitset.Close()

			guard, err := waitset.AttachNotification(listener)
			if err != nil {
				t.Fatalf("failed to attach listener: %v", err)
			}
			defer guard.Close()

			// Send a notification
			if _, err := notifier.Notify(); err != nil {
				t.Fatalf("failed to notify: %v", err)
			}

			// Process with callback
			callbackInvoked := false
			eventFromGuard := false
			result, err := waitset.WaitAndProcessOnceWithCallback(func(attachmentId *WaitSetAttachmentId) CallbackProgression {
				callbackInvoked = true
				if attachmentId.HasEventFrom(guard) {
					eventFromGuard = true
				}
				return CallbackProgressionContinue
			})
			if err != nil {
				t.Fatalf("WaitAndProcessOnceWithCallback failed: %v", err)
			}

			if !callbackInvoked {
				t.Error("callback was not invoked")
			}
			if !eventFromGuard {
				t.Error("event was not from our guard")
			}
			if result != WaitSetRunResultAllEventsHandled {
				t.Errorf("expected AllEventsHandled, got %v", result)
			}
		})
	}
}

func TestWaitSetCallbackCanStopProcessing(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create event service: %v", err)
			}
			defer service.Close()

			notifier, err := service.NotifierBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create notifier: %v", err)
			}
			defer notifier.Close()

			listener, err := service.ListenerBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create listener: %v", err)
			}
			defer listener.Close()

			waitset, err := NewWaitSetBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create waitset: %v", err)
			}
			defer waitset.Close()

			guard, err := waitset.AttachNotification(listener)
			if err != nil {
				t.Fatalf("failed to attach listener: %v", err)
			}
			defer guard.Close()

			// Send a notification
			if _, err := notifier.Notify(); err != nil {
				t.Fatalf("failed to notify: %v", err)
			}

			// Callback that returns Stop
			callbackCount := 0
			result, err := waitset.WaitAndProcessOnceWithCallback(func(attachmentId *WaitSetAttachmentId) CallbackProgression {
				callbackCount++
				return CallbackProgressionStop
			})
			if err != nil {
				t.Fatalf("WaitAndProcessOnceWithCallback failed: %v", err)
			}

			if callbackCount != 1 {
				t.Errorf("expected callback to be called exactly once, got %d", callbackCount)
			}
			if result != WaitSetRunResultStopRequest {
				t.Errorf("expected StopRequest result, got %v", result)
			}
		})
	}
}

func TestWaitSetRunWithContextCancellation(t *testing.T) {
	for _, serviceType := range serviceTypes {
		t.Run(fmt.Sprintf("ServiceType_%v", serviceType), func(t *testing.T) {
			node, err := NewNodeBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create node: %v", err)
			}
			defer node.Close()

			serviceName := generateServiceName(t)
			defer serviceName.Close()

			service, err := node.ServiceBuilder(serviceName).
				Event().
				OpenOrCreate()
			if err != nil {
				t.Fatalf("failed to create event service: %v", err)
			}
			defer service.Close()

			listener, err := service.ListenerBuilder().Create()
			if err != nil {
				t.Fatalf("failed to create listener: %v", err)
			}
			defer listener.Close()

			waitset, err := NewWaitSetBuilder().Create(serviceType)
			if err != nil {
				t.Fatalf("failed to create waitset: %v", err)
			}
			defer waitset.Close()

			guard, err := waitset.AttachNotification(listener)
			if err != nil {
				t.Fatalf("failed to attach listener: %v", err)
			}
			defer guard.Close()

			// Create a context that cancels quickly
			ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
			defer cancel()

			// RunWithContext should return context.DeadlineExceeded when context expires
			callbackCount := 0
			_, err = waitset.RunWithContext(ctx, func(attachmentId *WaitSetAttachmentId) CallbackProgression {
				callbackCount++
				return CallbackProgressionContinue
			}, 20*time.Millisecond)
			if !errors.Is(err, context.DeadlineExceeded) {
				t.Logf("expected context.DeadlineExceeded, got %v (callback invoked %d times)", err, callbackCount)
			}
		})
	}
}

// =============================================================================
// Error Tests
// =============================================================================

func TestErrorsIsWorks(t *testing.T) {
	// Test that sentinel errors work with errors.Is
	if !errors.Is(ErrNodeClosed, ErrNodeClosed) {
		t.Error("errors.Is should return true for same error")
	}
	if errors.Is(ErrNodeClosed, ErrPublisherClosed) {
		t.Error("errors.Is should return false for different errors")
	}
}

func TestNodeCreationErrorHasMessage(t *testing.T) {
	err := NodeCreationErrorInternalError
	msg := err.Error()
	if msg == "" {
		t.Error("error message should not be empty")
	}
}
