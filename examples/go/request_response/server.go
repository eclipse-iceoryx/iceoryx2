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

// This example demonstrates how to create a server in the request-response pattern.
// The server receives requests and sends responses.
package main

import (
	"errors"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"
	"unsafe"

	iox2 "github.com/eclipse-iceoryx/iceoryx2/iceoryx2-go/pkg/iceoryx2"
)

// Request represents a request message.
type Request struct {
	X int32
	Y int32
}

// Response represents a response message.
type Response struct {
	Sum int32
}

func main() {
	// Set up signal handling for graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	// Create a node
	node, err := iox2.NewNodeBuilder().
		Name("server").
		Create(iox2.ServiceTypeIpc)
	if err != nil {
		fmt.Printf("Failed to create node: %v\n", err)
		return
	}
	defer node.Close()

	fmt.Println("Server node created successfully")

	// Create the service name
	serviceName, err := iox2.NewServiceName("calculator/add")
	if err != nil {
		fmt.Printf("Failed to create service name: %v\n", err)
		return
	}
	defer serviceName.Close()

	// Create the request-response service
	service, err := node.ServiceBuilder(serviceName).
		RequestResponse().
		RequestPayloadType("Request", uint64(unsafe.Sizeof(Request{})), 4).
		ResponsePayloadType("Response", uint64(unsafe.Sizeof(Response{})), 4).
		OpenOrCreate()
	if err != nil {
		fmt.Printf("Failed to create service: %v\n", err)
		return
	}
	defer service.Close()

	fmt.Println("Request-Response service created successfully")

	// Create a server
	server, err := service.Server().Create()
	if err != nil {
		fmt.Printf("Failed to create server: %v\n", err)
		return
	}
	defer server.Close()

	fmt.Println("Server created successfully. Waiting for requests...")
	fmt.Println("Press Ctrl+C to exit")

	// Main loop
	running := true
	go func() {
		<-sigChan
		running = false
	}()

	for running {
		// Check for requests
		hasRequests, err := server.HasRequests()
		if err != nil {
			fmt.Printf("Error checking for requests: %v\n", err)
			time.Sleep(100 * time.Millisecond)
			continue
		}

		if hasRequests {
			// Receive the request
			activeRequest, err := server.Receive()
			if errors.Is(err, iox2.ErrNoData) {
				// Request was consumed by another process, continue
				continue
			}
			if err != nil {
				fmt.Printf("Error receiving request: %v\n", err)
				continue
			}

			// Get the request payload
			request := iox2.ActiveRequestPayloadAs[Request](activeRequest)
			if request != nil {
				fmt.Printf("Received request: %d + %d\n", request.X, request.Y)

				// Calculate the response
				response := Response{
					Sum: request.X + request.Y,
				}

				// Send the response
				err = iox2.ActiveRequestSendCopyAs(activeRequest, &response)
				if err != nil {
					fmt.Printf("Error sending response: %v\n", err)
				} else {
					fmt.Printf("Sent response: %d\n", response.Sum)
				}
			}

			activeRequest.Close()
		}

		time.Sleep(10 * time.Millisecond)
	}

	fmt.Println("\nServer shutting down...")
}
