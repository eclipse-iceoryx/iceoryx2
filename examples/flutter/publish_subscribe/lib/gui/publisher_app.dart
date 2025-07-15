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

class PublisherApp extends StatelessWidget {
  const PublisherApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'iceoryx2 Publisher',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        useMaterial3: true,
      ),
      home: const PublisherScreen(),
    );
  }
}

class PublisherScreen extends StatefulWidget {
  const PublisherScreen({super.key});

  @override
  State<PublisherScreen> createState() => _PublisherScreenState();
}

class _PublisherScreenState extends State<PublisherScreen> {
  Node? _node;
  Publisher? _publisher;
  final TextEditingController _messageController = TextEditingController();
  final List<String> _sentMessages = [];
  bool _isInitialized = false;
  String _status = 'Not initialized';
  Timer? _autoPublishTimer;
  bool _autoPublish = false;
  int _messageCounter = 0;

  @override
  void initState() {
    super.initState();
    _initializeIceoryx2();
  }

  void _initializeIceoryx2() async {
    try {
      print('[Publisher] Starting initialization...');
      setState(() {
        _status = 'Initializing...';
      });

      // Create node
      print('[Publisher] Creating iceoryx2 node...');
      _node = Node('iox2-flutter-publisher');
      print('[Publisher] OK Node created successfully');

      // Create publisher for service "flutter_example"
      print('[Publisher] Creating publisher for service "flutter_example"...');
      _publisher = _node!.publisher('flutter_example');
      print('[Publisher] OK Publisher created successfully');

      setState(() {
        _isInitialized = true;
        _status = 'Ready to publish';
      });
      print('[Publisher] Initialization completed successfully');
    } catch (e) {
      print('[Publisher] Initialization failed: $e');
      setState(() {
        _status = 'Error: $e';
      });
    }
  }

  void _publishMessage() {
    if (!_isInitialized || _publisher == null) {
      print('[Publisher] Cannot publish: not initialized or publisher is null');
      return;
    }

    final message = _messageController.text.trim();
    if (message.isEmpty) {
      print('[Publisher] Cannot publish: message is empty');
      return;
    }

    print('[Publisher] Publishing message: "$message"');
    try {
      _publisher!.sendText(message, sender: 'Flutter Publisher');
      print('[Publisher] OK Message published successfully');
      setState(() {
        _sentMessages.insert(
            0, '${DateTime.now().toIso8601String()}: $message');
        _status = 'Published: $message';
      });
      _messageController.clear();
    } catch (e) {
      print('[Publisher] ERROR Failed to publish message: $e');
      setState(() {
        _status = 'Error publishing: $e';
      });
    }
  }

  void _toggleAutoPublish() {
    setState(() {
      _autoPublish = !_autoPublish;
    });

    if (_autoPublish) {
      print('[Publisher] Starting auto-publish mode (every 2 seconds)');
      _autoPublishTimer = Timer.periodic(const Duration(seconds: 2), (timer) {
        _messageCounter++;
        final autoMessage = 'Auto message #$_messageCounter';

        print('[Publisher] Auto-publishing: "$autoMessage"');
        try {
          _publisher!.sendText(autoMessage, sender: 'Flutter Auto Publisher');
          print('[Publisher] OK Auto-message published successfully');
          setState(() {
            _sentMessages.insert(
                0, '${DateTime.now().toIso8601String()}: $autoMessage');
            _status = 'Auto-published: $autoMessage';
          });
        } catch (e) {
          print('[Publisher] ERROR Auto-publish error: $e');
          setState(() {
            _status = 'Auto-publish error: $e';
          });
        }
      });
    } else {
      print('[Publisher] Stopping auto-publish mode');
      _autoPublishTimer?.cancel();
    }
  }

  @override
  void dispose() {
    print('[Publisher] Starting cleanup...');
    _autoPublishTimer?.cancel();
    _messageController.dispose();

    // Clean up iceoryx2 resources
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

    print('[Publisher] Cleanup completed');
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('iceoryx2 Publisher'),
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

            // Message input
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Send Message',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    TextField(
                      controller: _messageController,
                      decoration: const InputDecoration(
                        hintText: 'Enter message to publish...',
                        border: OutlineInputBorder(),
                      ),
                      enabled: _isInitialized,
                      onSubmitted: (_) => _publishMessage(),
                    ),
                    const SizedBox(height: 8),
                    Row(
                      children: [
                        ElevatedButton(
                          onPressed: _isInitialized ? _publishMessage : null,
                          child: const Text('Publish'),
                        ),
                        const SizedBox(width: 8),
                        ElevatedButton(
                          onPressed: _isInitialized ? _toggleAutoPublish : null,
                          style: ElevatedButton.styleFrom(
                            backgroundColor:
                                _autoPublish ? Colors.red : Colors.green,
                            foregroundColor: Colors.white,
                          ),
                          child:
                              Text(_autoPublish ? 'Stop Auto' : 'Start Auto'),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Sent messages list
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
                            'Sent Messages (${_sentMessages.length})',
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          if (_sentMessages.isNotEmpty)
                            TextButton(
                              onPressed: () {
                                setState(() {
                                  _sentMessages.clear();
                                });
                              },
                              child: const Text('Clear'),
                            ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      Expanded(
                        child: _sentMessages.isEmpty
                            ? const Center(
                                child: Text('No messages sent yet'),
                              )
                            : ListView.builder(
                                itemCount: _sentMessages.length,
                                itemBuilder: (context, index) {
                                  return Card(
                                    margin:
                                        const EdgeInsets.symmetric(vertical: 2),
                                    child: ListTile(
                                      dense: true,
                                      leading: const Icon(Icons.send, size: 16),
                                      title: Text(
                                        _sentMessages[index],
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
