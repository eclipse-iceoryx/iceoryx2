// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

/// iceoryx2 Flutter Package
///
/// This library provides a high-level Dart API for iceoryx2 inter-process communication.
///
/// ## Quick Start
///
/// ```dart
/// import 'package:iceoryx2_flutter_examples/iceoryx2.dart';
///
/// // Create a node
/// final node = Node('my-app');
///
/// // Create a publisher
/// final publisher = node.publisher('my-service');
/// publisher.sendText('Hello, World!');
///
/// // Create a subscriber
/// final subscriber = node.subscriber('my-service');
/// subscriber.messages.listen((message) {
///   print('Received: ${message.content}');
/// });
///
/// // Clean up
/// publisher.close();
/// subscriber.close();
/// node.close();
/// ```
library iceoryx2;

// Export public API
export 'src/iceoryx2_api.dart'
    show Node, Publisher, Subscriber, Iceoryx2Exception;
export 'src/message_protocol.dart' show Message, MessageProtocol;

// Private implementations are not exported:
// - src/ffi/iceoryx2_ffi.dart (FFI bindings)
// - Legacy MessageHelper (deprecated)
