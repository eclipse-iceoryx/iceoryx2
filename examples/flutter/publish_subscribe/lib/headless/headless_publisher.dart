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

import 'dart:io';
import 'dart:async';
import '../iceoryx2.dart';

/// Headless publisher for testing - sends messages automatically
/// No UI, pure console output for validation
void main() async {
  print('=== Headless iceoryx2 Publisher Test ===');
  print('Starting automatic message publisher...');

  final publisher = HeadlessPublisher();
  await publisher.initialize();

  // Handle graceful shutdown
  ProcessSignal.sigint.watch().listen((signal) {
    print('
Received SIGINT, shutting down gracefully...');
    publisher.stop();
    exit(0);
  });

  // Start publishing messages
  await publisher.startAutoPublish();

  // Keep main thread alive
  while (true) {
    await Future.delayed(const Duration(seconds: 1));
  }
}

class HeadlessPublisher {
  Node? _node;
  Publisher? _publisher;
  Timer? _publishTimer;
  int _messageCounter = 0;
  int _successCount = 0;
  int _failureCount = 0;
  DateTime? _startTime;

  Future<void> initialize() async {
    try {
      print('[Publisher] Initializing iceoryx2...');

      // Create node
      print('[Publisher] Creating iceoryx2 node...');
      _node = Node('iox2-flutter-headless-pub');
      print('[Publisher] OK Node created successfully');

      // Create publisher for service "flutter_example"
      print('[Publisher] Creating publisher for service "flutter_example"...');
      _publisher = _node!.publisher('flutter_example');
      print('[Publisher] OK Publisher created successfully');

      print('[Publisher] OK Initialization completed');
    } catch (e) {
      print('[Publisher] ERROR Initialization failed: $e');
      rethrow;
    }
  }

  Future<void> startAutoPublish() async {
    if (_publisher == null) {
      throw Exception('Publisher not initialized');
    }

    print('[Publisher] Starting automatic publishing (every 2 seconds)...');
    _startTime = DateTime.now();

    _publishTimer = Timer.periodic(const Duration(seconds: 2), (timer) {
      _sendMessage();
    });

    // Send first message immediately
    _sendMessage();
  }

  void _sendMessage() {
    if (_publisher == null) return;

    _messageCounter++;
    final message = 'Headless message #$_messageCounter';

    try {
      _publisher!.sendText(message, sender: 'Headless Publisher');
      _successCount++;
      print('[Publisher] OK Sent message #$_messageCounter: "$message"');

      // Print periodic stats
      if (_messageCounter % 10 == 0) {
        _printStats();
      }
    } catch (e) {
      _failureCount++;
      print('[Publisher] ERROR Failed to send message #$_messageCounter: $e');
    }
  }

  void _printStats() {
    final elapsed = _startTime != null
        ? DateTime.now().difference(_startTime!).inSeconds
        : 0;

    print('[Publisher] === Statistics ===');
    print('[Publisher] Total messages: $_messageCounter');
    print('[Publisher] Successful: $_successCount');
    print('[Publisher] Failed: $_failureCount');
    print(
        '[Publisher] Success rate: ${(_successCount / _messageCounter * 100).toStringAsFixed(1)}%');
    print('[Publisher] Elapsed time: ${elapsed}s');
    print(
        '[Publisher] Messages/second: ${(_messageCounter / elapsed).toStringAsFixed(2)}');
    print('[Publisher] ====================');
  }

  void stop() {
    print('[Publisher] Stopping publisher...');
    _publishTimer?.cancel();

    _printStats();

    if (_publisher != null) {
      print('[Publisher] Closing publisher...');
      _publisher!.close();
      print('[Publisher] OK Publisher closed');
    }

    if (_node != null) {
      print('[Publisher] Closing node...');
      _node!.close();
      print('[Publisher] OK Node closed');
    }

    print('[Publisher] OK Stopped successfully');
  }
}
