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

/// Headless subscriber for testing - receives messages automatically
/// No UI, pure console output for validation
void main() async {
  print('=== Headless iceoryx2 Subscriber Test ===');
  print('Starting automatic message subscriber...');

  final subscriber = HeadlessSubscriber();
  await subscriber.initialize();

  // Handle graceful shutdown
  ProcessSignal.sigint.watch().listen((signal) {
    print('
Received SIGINT, shutting down gracefully...');
    subscriber.stop();
    exit(0);
  });

  // Start listening for messages
  await subscriber.startListening();

  // Keep main thread alive
  while (true) {
    await Future.delayed(const Duration(seconds: 1));
  }
}

class HeadlessSubscriber {
  Node? _node;
  Subscriber? _subscriber;
  StreamSubscription<Message>? _messageSubscription;
  int _totalReceived = 0;
  DateTime? _startTime;
  bool _isListening = false;

  Future<void> initialize() async {
    try {
      print('[Subscriber] Initializing iceoryx2...');

      // Create node
      print('[Subscriber] Creating iceoryx2 node...');
      _node = Node('iox2-flutter-headless-sub');
      print('[Subscriber] OK Node created successfully');

      // Create subscriber for service "flutter_example"
      print(
          '[Subscriber] Creating subscriber for service "flutter_example"...');
      _subscriber = _node!.subscriber('flutter_example');
      print('[Subscriber] OK Subscriber created successfully');

      print('[Subscriber] OK Initialization completed');
    } catch (e) {
      print('[Subscriber] ERROR Initialization failed: $e');
      rethrow;
    }
  }

  Future<void> startListening() async {
    if (_subscriber == null || _node == null) {
      throw Exception('Subscriber or node not initialized');
    }

    print(
        '[Subscriber] Starting event-driven message listening with waitset...');
    _isListening = true;
    _startTime = DateTime.now();

    // Start the efficient waitset-based listening
    _subscriber!.startListening();

    // Start listening to messages using the high-level API
    _messageSubscription = _subscriber!.messages.listen(
      (message) {
        _totalReceived++;
        final now = DateTime.now();
        final elapsed =
            _startTime != null ? now.difference(_startTime!).inMilliseconds : 0;

        print('[Subscriber] OK #$_totalReceived: "${message.content}" '
            'from ${message.sender} (${elapsed}ms)');

        // Show periodic statistics
        if (_totalReceived % 10 == 0) {
          _showStatistics();
        }
      },
      onError: (error) {
        print('[Subscriber] ERROR Error in message stream: $error');
      },
      onDone: () {
        print('[Subscriber] Message stream closed');
        _isListening = false;
      },
    );

    // Also demonstrate manual polling in parallel
    _startManualPolling();
  }

  void _startManualPolling() {
    // Start a timer to occasionally try manual message reception
    Timer.periodic(const Duration(seconds: 5), (timer) {
      if (!_isListening) {
        timer.cancel();
        return;
      }

      try {
        final message = _subscriber!.tryReceive();
        if (message != null) {
          print(
              '[Subscriber] OK Manual poll received: "${message.content}" from ${message.sender}');
        } else {
          print('[Subscriber] Manual poll: no message available');
        }
      } catch (e) {
        print('[Subscriber] ERROR Manual poll error: $e');
      }
    });
  }

  void _showStatistics() {
    if (_startTime == null) return;

    final elapsed = DateTime.now().difference(_startTime!);
    final rate = _totalReceived / elapsed.inSeconds;

    print('[Subscriber] === Statistics ===');
    print('[Subscriber] Total received: $_totalReceived messages');
    print('[Subscriber] Elapsed time: ${elapsed.inSeconds}s');
    print('[Subscriber] Average rate: ${rate.toStringAsFixed(2)} msg/s');
    print('[Subscriber] ========================');
  }

  void stop() {
    print('[Subscriber] Stopping headless subscriber...');
    _isListening = false;
    _messageSubscription?.cancel();
    _messageSubscription = null;

    try {
      _subscriber?.close();
      _node?.close();
      print('[Subscriber] OK Cleanup completed');
    } catch (e) {
      print('[Subscriber] ERROR Cleanup error: $e');
    }

    if (_startTime != null) {
      _showStatistics();
    }
  }
}
