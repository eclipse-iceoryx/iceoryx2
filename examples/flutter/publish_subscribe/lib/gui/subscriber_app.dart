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
import 'package:flutter/material.dart';
import '../iceoryx2.dart';

class SubscriberApp extends StatelessWidget {
  const SubscriberApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'iceoryx2 Subscriber',
      theme: ThemeData(
        primarySwatch: Colors.green,
        useMaterial3: true,
      ),
      home: const SubscriberScreen(),
    );
  }
}

class SubscriberScreen extends StatefulWidget {
  const SubscriberScreen({super.key});

  @override
  State<SubscriberScreen> createState() => _SubscriberScreenState();
}

class _SubscriberScreenState extends State<SubscriberScreen> {
  Node? _node;
  Subscriber? _subscriber;
  final List<String> _receivedMessages = [];
  bool _isInitialized = false;
  bool _isListening = false;
  String _status = 'Not initialized';
  StreamSubscription<Message>? _messageSubscription;

  @override
  void initState() {
    super.initState();
    _initializeIceoryx2();
  }

  void _initializeIceoryx2() async {
    try {
      print('[Subscriber] Starting initialization...');
      setState(() {
        _status = 'Initializing...';
      });

      // Create node
      print('[Subscriber] Creating iceoryx2 node...');
      _node = Node('iox2-flutter-subscriber');
      print('[Subscriber] OK Node created successfully');

      // Create subscriber for service "flutter_example"
      print(
          '[Subscriber] Creating subscriber for service "flutter_example"...');
      _subscriber = _node!.subscriber('flutter_example');
      print('[Subscriber] OK Subscriber created successfully');

      setState(() {
        _isInitialized = true;
        _status = 'Ready to receive messages';
      });
      print('[Subscriber] OK Initialization completed successfully');
    } catch (e) {
      print('[Subscriber] ERROR Initialization failed: $e');
      setState(() {
        _status = 'Error: $e';
      });
    }
  }

  void _startListening() {
    if (!_isInitialized || _subscriber == null) {
      print(
          '[Subscriber] Cannot start listening: not initialized or subscriber is null');
      return;
    }

    print('[Subscriber] Starting event-driven message listening...');
    setState(() {
      _isListening = true;
      _status = 'Listening for messages...';
    });

    _messageSubscription = _subscriber!.messages.listen(
      (message) {
        print(
            '[Subscriber] OK Received message: "${message.content}" from ${message.sender}');
        setState(() {
          _receivedMessages.insert(0,
              '${message.timestamp.toIso8601String()}: ${message.content} (from: ${message.sender})');
          _status = 'Received: ${message.content}';
        });
      },
      onError: (error) {
        print('[Subscriber] ERROR Error in message stream: $error');
        setState(() {
          _status = 'Error receiving: $error';
        });
      },
      onDone: () {
        print('[Subscriber] Message stream closed');
        setState(() {
          _isListening = false;
          _status = 'Message stream closed';
        });
      },
    );
  }

  void _stopListening() {
    print('[Subscriber] Stopping message listener...');
    setState(() {
      _isListening = false;
      _status = 'Stopped listening';
    });
    _messageSubscription?.cancel();
    _messageSubscription = null;
  }

  void _tryReceiveOnce() {
    if (!_isInitialized || _subscriber == null) {
      print(
          '[Subscriber] Cannot receive: not initialized or subscriber is null');
      return;
    }

    print('[Subscriber] Trying to receive one message...');
    try {
      final message = _subscriber!.tryReceive();
      if (message != null) {
        print(
            '[Subscriber] OK Received message: "${message.content}" from ${message.sender}');
        setState(() {
          _receivedMessages.insert(0,
              '${message.timestamp.toIso8601String()}: ${message.content} (from: ${message.sender})');
          _status = 'Manually received: ${message.content}';
        });
      } else {
        print('[Subscriber] No message available');
        setState(() {
          _status = 'No message available';
        });
      }
    } catch (e) {
      print('[Subscriber] ERROR Error receiving message: $e');
      setState(() {
        _status = 'Error receiving: $e';
      });
    }
  }

  @override
  void dispose() {
    print('[Subscriber] Starting cleanup...');
    _messageSubscription?.cancel();

    // Clean up iceoryx2 resources
    if (_subscriber != null) {
      print('[Subscriber] Closing subscriber...');
      _subscriber!.close();
      print('[Subscriber] OK Subscriber closed');
    }
    if (_node != null) {
      print('[Subscriber] Closing node...');
      _node!.close();
      print('[Subscriber] OK Node closed');
    }

    print('[Subscriber] OK Cleanup completed');
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('iceoryx2 Subscriber'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Status indicator
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Status',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Row(
                      children: [
                        Icon(
                          _isInitialized ? Icons.check_circle : Icons.error,
                          color: _isInitialized ? Colors.green : Colors.red,
                        ),
                        const SizedBox(width: 8),
                        Expanded(child: Text(_status)),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Control buttons
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Message Receiving',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Row(
                      children: [
                        ElevatedButton(
                          onPressed: _isInitialized && !_isListening
                              ? _startListening
                              : null,
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.green,
                            foregroundColor: Colors.white,
                          ),
                          child: const Text('Start Listening'),
                        ),
                        const SizedBox(width: 8),
                        ElevatedButton(
                          onPressed: _isListening ? _stopListening : null,
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.red,
                            foregroundColor: Colors.white,
                          ),
                          child: const Text('Stop Listening'),
                        ),
                        const SizedBox(width: 8),
                        ElevatedButton(
                          onPressed: _isInitialized && !_isListening
                              ? _tryReceiveOnce
                              : null,
                          child: const Text('Try Receive Once'),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Row(
                      children: [
                        Icon(
                          _isListening
                              ? Icons.radio_button_checked
                              : Icons.radio_button_off,
                          color: _isListening ? Colors.green : Colors.grey,
                        ),
                        const SizedBox(width: 8),
                        Text(_isListening
                            ? 'Actively listening'
                            : 'Not listening'),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Received messages list
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(
                            'Received Messages (${_receivedMessages.length})',
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          if (_receivedMessages.isNotEmpty)
                            TextButton(
                              onPressed: () {
                                setState(() {
                                  _receivedMessages.clear();
                                });
                              },
                              child: const Text('Clear'),
                            ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      Expanded(
                        child: _receivedMessages.isEmpty
                            ? const Center(
                                child: Text('No messages received yet'),
                              )
                            : ListView.builder(
                                itemCount: _receivedMessages.length,
                                itemBuilder: (context, index) {
                                  return Card(
                                    margin:
                                        const EdgeInsets.symmetric(vertical: 2),
                                    child: ListTile(
                                      dense: true,
                                      leading:
                                          const Icon(Icons.message, size: 16),
                                      title: Text(
                                        _receivedMessages[index],
                                        style: const TextStyle(fontSize: 12),
                                      ),
                                    ),
                                  );
                                },
                              ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
