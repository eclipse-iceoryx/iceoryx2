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

// This example demonstrates how to create a publisher for publish-subscribe communication.
// Run the subscriber first, then run this publisher in a separate terminal.
package main

import (
	"fmt"
	"log"
	"time"
	"unsafe"

	iceoryx2 "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

// TransmissionData is the payload type being published.
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
		Name("publisher-node").
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

	// Create publisher
	publisher, err := service.PublisherBuilder().Create()
	if err != nil {
		log.Fatalf("Unable to create publisher: %v", err)
	}
	defer publisher.Close()

	fmt.Println("Publisher ready to publish!")

	counter := int32(0)
	for {
		// Loan a sample and write data
		sample, err := publisher.LoanUninit()
		if err != nil {
			log.Printf("Unable to loan sample: %v", err)
			time.Sleep(time.Second)
			continue
		}

		// Write payload data using PayloadMutAs generic function
		payload := iceoryx2.PayloadMutAs[TransmissionData](sample)
		payload.X = counter
		payload.Y = counter * 2
		payload.Funky = float64(counter) * 1.234

		fmt.Printf("Publishing: x=%d, y=%d, funky=%.3f\n", payload.X, payload.Y, payload.Funky)

		// Send the sample
		err = sample.Send()
		if err != nil {
			log.Printf("Unable to send sample: %v", err)
		}

		counter++
		time.Sleep(time.Second)

		// Check for termination signal
		if err := node.Wait(0); err != nil {
			fmt.Println("Received termination signal")
			break
		}
	}
}
