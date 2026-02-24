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

// This example demonstrates how to create a notifier to send events.
// Use this together with the waitset example.
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
		Name("notifier-for-waitset").
		Create(iox2.ServiceTypeIpc)
	if err != nil {
		fmt.Printf("Failed to create node: %v\n", err)
		return
	}
	defer node.Close()

	fmt.Println("Node created successfully")

	// Open the event service
	serviceName, err := iox2.NewServiceName("waitset/demo")
	if err != nil {
		fmt.Printf("Failed to create service name: %v\n", err)
		return
	}
	defer serviceName.Close()

	service, err := node.ServiceBuilder(serviceName).
		Event().
		Open()
	if err != nil {
		fmt.Printf("Failed to open event service: %v\n", err)
		fmt.Println("Make sure the waitset example is running first!")
		return
	}
	defer service.Close()

	fmt.Println("Event service opened successfully")

	// Create a notifier
	notifier, err := service.NotifierBuilder().Create()
	if err != nil {
		fmt.Printf("Failed to create notifier: %v\n", err)
		return
	}
	defer notifier.Close()

	fmt.Println("Notifier created successfully")
	fmt.Println("Sending events every second...")
	fmt.Println("Press Ctrl+C to exit")

	// Main loop
	running := true
	go func() {
		<-sigChan
		running = false
	}()

	eventId := uint64(1)
	for running {
		fmt.Printf("Sending event with ID: %d\n", eventId)

		_, err := notifier.NotifyWithEventId(eventId)
		if err != nil {
			fmt.Printf("Error sending notification: %v\n", err)
		}

		eventId++
		time.Sleep(time.Second)
	}

	fmt.Println("\nNotifier shutting down...")
}
