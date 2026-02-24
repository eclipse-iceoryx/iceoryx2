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

// Package iceoryx2 provides Go bindings for the iceoryx2 inter-process communication library.
//
// iceoryx2 is a high-performance, lock-free, and zero-copy inter-process communication (IPC)
// library that provides publish-subscribe, event, and request-response messaging patterns.
//
// # Getting Started
//
// Create a node, which is the central entry point:
//
//	node, err := iceoryx2.NewNodeBuilder().
//	    Name("my-app").
//	    Create(iceoryx2.ServiceTypeIpc)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer node.Close()
//
// # Publish-Subscribe Pattern
//
// Publisher:
//
//	serviceName, _ := iceoryx2.NewServiceName("My/Funk/ServiceName")
//	service, _ := node.ServiceBuilder(serviceName).
//	    PublishSubscribe().
//	    OpenOrCreate()
//	defer service.Close()
//
//	publisher, _ := service.PublisherBuilder().Create()
//	defer publisher.Close()
//
//	sample, _ := publisher.LoanUninit()
//	// Write payload...
//	sample.Send()
//
// Subscriber:
//
//	subscriber, _ := service.SubscriberBuilder().Create()
//	defer subscriber.Close()
//
//	sample, _ := subscriber.Receive()
//	if sample != nil {
//	    payload := sample.Payload()
//	    // Process payload...
//	    sample.Close()
//	}
//
// # Event Pattern
//
// Notifier:
//
//	service, _ := node.ServiceBuilder(serviceName).
//	    Event().
//	    OpenOrCreate()
//
//	notifier, _ := service.NotifierBuilder().Create()
//	notifier.Notify()
//
// Listener:
//
//	listener, _ := service.ListenerBuilder().Create()
//	listener.WaitAll(func(eventId EventId) CallbackProgression {
//	    // Handle event...
//	    return CallbackProgressionContinue
//	})
package iceoryx2
