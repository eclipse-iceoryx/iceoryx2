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

import 'dart:async';
import 'package:iceoryx2_flutter_examples/iceoryx2.dart';

/// Test polling-based message receiving
void main() async {
  print('=== Polling-based Subscriber Test ===');

  Node? node;
  Subscriber? subscriber;

  try {
    // Create node and subscriber
    print('[Test] Creating node and subscriber...');
    node = Node('polling-test-node');
    subscriber = node.subscriber('flutter_example');
    print('[Test] OK Node and subscriber created');

    print('[Test] Testing polling for 10 seconds...');
    final startTime = DateTime.now();
    int messageCount = 0;

    // Poll for messages for 10 seconds
    while (DateTime.now().difference(startTime).inSeconds < 10) {
      final message = subscriber.tryReceive();
      if (message != null) {
        messageCount++;
        print(
            '[Test] OK Received message #$messageCount: "${message.content}" from ${message.sender}');
      } else {
        // Wait a bit before next poll
        await Future.delayed(const Duration(milliseconds: 100));
      }
    }

    print('[Test] OK Polling completed. Received $messageCount messages.');
  } catch (e) {
    print('[Test] ERROR Error: $e');
  } finally {
    // Clean up
    subscriber?.close();
    node?.close();
    print('[Test] OK Resources cleaned up');
  }
}
