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

// This example demonstrates how to create a notifier for event-based communication.
// Run the listener first, then run this notifier in a separate terminal.
package main

import (
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
		Name("notifier-node").
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

	// Open event service
	service, err := node.ServiceBuilder(serviceName).
		Event().
		Open()
	if err != nil {
		log.Fatalf("Unable to open service: %v", err)
	}
	defer service.Close()

	// Create notifier
	notifier, err := service.NotifierBuilder().Create()
	if err != nil {
		log.Fatalf("Unable to create notifier: %v", err)
	}
	defer notifier.Close()

	fmt.Println("Notifier ready to send events!")

	eventId := uint64(0)
	for {
		fmt.Printf("Triggering event with id: %d\n", eventId)

		_, err := notifier.NotifyWithEventId(eventId)
		if err != nil {
			log.Printf("Failed to notify: %v", err)
		}

		eventId++
		time.Sleep(time.Second)

		// Check for termination signal
		if err := node.Wait(0); err != nil {
			fmt.Println("Received termination signal")
			break
		}
	}
}
