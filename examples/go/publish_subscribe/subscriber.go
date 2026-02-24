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

// This example demonstrates how to create a subscriber for publish-subscribe communication.
// Run this subscriber first, then run the publisher in a separate terminal.
package main

import (
	"errors"
	"fmt"
	"log"
	"time"
	"unsafe"

	iceoryx2 "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

// TransmissionData is the payload type being received.
type TransmissionData struct {
	X     int32
	Y     int32
	Funky float64
}

func main() {
	// Setup logging
	iceoryx2.SetLogLevelFromEnvOr(iceoryx2.LogLevelInfo)

	// Create new node
	node, err := iceoryx2.NewNodeBuilder().
		Name("subscriber-node").
		Create(iceoryx2.ServiceTypeIpc)
	if err != nil {
		log.Fatalf("Could not create node: %v", err)
	}
	defer node.Close()

	// Create service name
	serviceName, err := iceoryx2.NewServiceName("My/Funky/Service")
	if err != nil {
		log.Fatalf("Unable to create service name: %v", err)
	}
	defer serviceName.Close()

	// Create publish-subscribe service
	service, err := node.ServiceBuilder(serviceName).
		PublishSubscribe().
		PayloadType("TransmissionData", uint64(unsafe.Sizeof(TransmissionData{})), uint64(unsafe.Alignof(TransmissionData{}))).
		OpenOrCreate()
	if err != nil {
		log.Fatalf("Unable to create service: %v", err)
	}
	defer service.Close()

	// Create subscriber
	subscriber, err := service.SubscriberBuilder().Create()
	if err != nil {
		log.Fatalf("Unable to create subscriber: %v", err)
	}
	defer subscriber.Close()

	fmt.Println("Subscriber waiting for messages...")

	for {
		// Try to receive a sample
		sample, err := subscriber.Receive()
		if errors.Is(err, iceoryx2.ErrNoData) {
			// No sample available yet, continue polling
			time.Sleep(100 * time.Millisecond)
			continue
		}
		if err != nil {
			log.Printf("Receive error: %v", err)
			time.Sleep(time.Second)
			continue
		}

		// Access the payload using PayloadAs generic function
		payload := iceoryx2.PayloadAs[TransmissionData](sample)
		fmt.Printf("Received: x=%d, y=%d, funky=%.3f\n", payload.X, payload.Y, payload.Funky)
		// Release the sample after use
		sample.Close()
	}
}
