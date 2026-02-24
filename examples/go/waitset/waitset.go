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

//go:build ignore

// This example demonstrates how to use a WaitSet to wait for events.
// The WaitSet can attach to listeners and wait for notifications, deadlines, or intervals.
package main

import (
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	iox2 "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

func main() {
	// Set up signal handling for graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	// Create a node
	node, err := iox2.NewNodeBuilder().
		Name("waitset-example").
		Create(iox2.ServiceTypeIpc)
	if err != nil {
		fmt.Printf("Failed to create node: %v\n", err)
		return
	}
	defer node.Close()

	fmt.Println("Node created successfully")

	// Create an event service
	serviceName, err := iox2.NewServiceName("waitset/demo")
	if err != nil {
		fmt.Printf("Failed to create service name: %v\n", err)
		return
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		Event().
		OpenOrCreate()
	if err != nil {
		fmt.Printf("Failed to create event service: %v\n", err)
		return
	}
	defer service.Close()

	fmt.Println("Event service created successfully")

	// Create a listener
	listener, err := service.ListenerBuilder().Create()
	if err != nil {
		fmt.Printf("Failed to create listener: %v\n", err)
		return
	}
	defer listener.Close()

	fmt.Println("Listener created successfully")

	// Create a WaitSet
	waitset, err := iox2.NewWaitSetBuilder().
		SignalHandlingMode(iox2.SignalHandlingModeHandleTerminationRequests).
		Create(iox2.ServiceTypeIpc)
	if err != nil {
		fmt.Printf("Failed to create waitset: %v\n", err)
		return
	}
	defer waitset.Close()

	fmt.Println("WaitSet created successfully")

	// Attach the listener to the waitset for notification events
	listenerGuard, err := waitset.AttachNotification(listener)
	if err != nil {
		fmt.Printf("Failed to attach listener: %v\n", err)
		return
	}
	defer listenerGuard.Close()

	fmt.Println("Listener attached to WaitSet")

	// Attach an interval timer (triggers every 2 seconds)
	intervalGuard, err := waitset.AttachInterval(2 * time.Second)
	if err != nil {
		fmt.Printf("Failed to attach interval: %v\n", err)
		return
	}
	defer intervalGuard.Close()

	fmt.Println("Interval timer (2s) attached to WaitSet")
	fmt.Printf("WaitSet has %d attachments, capacity: %d\n",
		waitset.NumberOfAttachments(), waitset.Capacity())

	fmt.Println("\nWaiting for events...")
	fmt.Println("Run 'go run notifier.go' in another terminal to send events")
	fmt.Println("Press Ctrl+C to exit")

	// Main loop
	running := true
	go func() {
		<-sigChan
		running = false
	}()

	eventCount := 0
	for running {
		// Wait for events with a timeout
		result, err := waitset.WaitAndProcessOnceWithTimeout(1 * time.Second)
		if err != nil {
			if running {
				fmt.Printf("WaitSet error: %v\n", err)
			}
			continue
		}

		switch result {
		case iox2.WaitSetRunResultTerminationRequest:
			fmt.Println("Termination requested")
			running = false
		case iox2.WaitSetRunResultInterrupt:
			fmt.Println("Interrupted")
		case iox2.WaitSetRunResultStopRequest:
			fmt.Println("Stop requested")
			running = false
		case iox2.WaitSetRunResultAllEventsHandled:
			eventCount++
			fmt.Printf("[%d] Events processed\n", eventCount)

			// After the waitset processes events, try to receive from the listener
			for {
				eventId, err := listener.TryWaitOne()
				if err != nil || eventId == nil {
					break
				}
				fmt.Printf("  Received Event ID: %d\n", *eventId)
			}
		}
	}

	fmt.Println("\nWaitSet example shutting down...")
}
