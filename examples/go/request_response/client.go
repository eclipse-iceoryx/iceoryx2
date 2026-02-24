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

// This example demonstrates how to create a client in the request-response pattern.
// The client sends requests and receives responses.
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
		Name("client").
		Create(iox2.ServiceTypeIpc)
	if err != nil {
		fmt.Printf("Failed to create node: %v\n", err)
		return
	}
	defer node.Close()

	fmt.Println("Client node created successfully")

	// Create the service name
	serviceName, err := iox2.NewServiceName("calculator/add")
	if err != nil {
		fmt.Printf("Failed to create service name: %v\n", err)
		return
	}
	defer serviceName.Close()

	// Open the request-response service
	service, err := node.ServiceBuilder(serviceName).
		RequestResponse().
		RequestPayloadType("Request", uint64(unsafe.Sizeof(Request{})), 4).
		ResponsePayloadType("Response", uint64(unsafe.Sizeof(Response{})), 4).
		Open()
	if err != nil {
		fmt.Printf("Failed to open service: %v\n", err)
		return
	}
	defer service.Close()

	fmt.Println("Request-Response service opened successfully")

	// Create a client
	client, err := service.Client().Create()
	if err != nil {
		fmt.Printf("Failed to create client: %v\n", err)
		return
	}
	defer client.Close()

	fmt.Println("Client created successfully")
	fmt.Println("Press Ctrl+C to exit")

	// Main loop - send requests and receive responses
	running := true
	go func() {
		<-sigChan
		running = false
	}()

	x := int32(1)
	y := int32(2)

	for running {
		// Create a request
		request := Request{
			X: x,
			Y: y,
		}

		fmt.Printf("Sending request: %d + %d\n", request.X, request.Y)

		// Send the request
		pendingResponse, err := iox2.SendCopyAs(client, &request)
		if err != nil {
			fmt.Printf("Error sending request: %v\n", err)
			time.Sleep(time.Second)
			continue
		}

		// Wait for response
		responseReceived := false
		for !responseReceived && running {
			response, err := pendingResponse.Receive()
			if errors.Is(err, iox2.ErrNoData) {
				// No response yet, wait a bit
				time.Sleep(10 * time.Millisecond)
				continue
			}
			if err != nil {
				fmt.Printf("Error receiving response: %v\n", err)
				break
			}

			responseData := iox2.ResponsePayloadAs[Response](response)
			if responseData != nil {
				fmt.Printf("Received response: %d\n", responseData.Sum)
			}
			response.Close()
			responseReceived = true
		}

		pendingResponse.Close()

		// Increment values for next request
		x++
		y++

		time.Sleep(time.Second)
	}

	fmt.Println("\nClient shutting down...")
}
