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

// This example demonstrates how to create a listener for event-based communication.
// Run this listener first, then run the notifier in a separate terminal.
package main

import (
	"errors"
	"fmt"
	"log"
	"time"

	iceoryx2 "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

func main() {
	// Setup logging
	iceoryx2.SetLogLevelFromEnvOr(iceoryx2.LogLevelInfo)

	// Create new node
	node, err := iceoryx2.NewNodeBuilder().
		Name("listener-node").
		Create(iceoryx2.ServiceTypeIpc)
	if err != nil {
		log.Fatalf("Could not create node: %v", err)
	}
	defer node.Close()

	// Create service name
	serviceName, err := iceoryx2.NewServiceName("MyEventName")
	if err != nil {
		log.Fatalf("Unable to create service name: %v", err)
	}
	defer serviceName.Close()

	// Create event service
	service, err := node.ServiceBuilder(serviceName).
		Event().
		OpenOrCreate()
	if err != nil {
		log.Fatalf("Unable to create service: %v", err)
	}
	defer service.Close()

	// Create listener
	listener, err := service.ListenerBuilder().Create()
	if err != nil {
		log.Fatalf("Unable to create listener: %v", err)
	}
	defer listener.Close()

	fmt.Println("Listener ready to receive events!")

	for {
		// Try non-blocking wait first
		eventId, err := listener.TryWaitOne()
		if errors.Is(err, iceoryx2.ErrNoData) {
			// No event, wait a bit to avoid busy-waiting
			time.Sleep(100 * time.Millisecond)
		} else if err != nil {
			log.Printf("Unable to wait for notification: %v", err)
			break
		} else {
			fmt.Printf("Event was triggered with id: %d\n", *eventId)
		}

		// Check for termination signal
		if err := node.Wait(0); err != nil {
			fmt.Println("Received termination signal")
			break
		}
	}
}
