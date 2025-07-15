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

/// High-level Dart API for iceoryx2
/// This file provides high-level classes that wrap FFI for easy use by Dart developers.
library iceoryx2_api;

import 'dart:ffi';
import 'dart:async';
import 'dart:isolate';
import 'dart:io';
import 'package:ffi/ffi.dart';

import 'ffi/iceoryx2_ffi.dart' as ffi;
import 'message_protocol.dart';

/// Exception thrown when iceoryx2 operations fail
class Iceoryx2Exception implements Exception {
  final String message;
  final int? errorCode;

  const Iceoryx2Exception(this.message, [this.errorCode]);

  @override
  String toString() => errorCode != null
      ? 'Iceoryx2Exception: $message (code: $errorCode)'
      : 'Iceoryx2Exception: $message';
}

/// Manages the lifecycle of an iceoryx2 node
class Node implements Finalizable {
  late final Pointer<ffi.Iox2Node> _handle;
  bool _isClosed = false;
  final String _name;

  /// Create a new iceoryx2 node with the given name
  Node(this._name) {
    try {
      print('[Node] Creating node: "$_name"');

      final nodeBuilder = ffi.iox2NodeBuilderNew(nullptr);
      if (nodeBuilder == nullptr) {
        throw const Iceoryx2Exception('Failed to create node builder');
      }

      final nodePointer = calloc<Pointer<ffi.Iox2Node>>();

      final result = ffi.iox2NodeBuilderCreate(
          nodeBuilder, nullptr, ffi.IOX2_SERVICE_TYPE_IPC, nodePointer.cast());

      if (result != ffi.IOX2_OK) {
        calloc.free(nodePointer);
        throw Iceoryx2Exception('Failed to create node', result);
      }

      _handle = nodePointer.value;
      calloc.free(nodePointer);

      _finalizer.attach(this, _handle.cast(), detach: this);
      print('[Node] OK Node "$_name" created successfully');
    } catch (e) {
      print('[Node] ERROR Failed to create node "$_name": $e');
      rethrow;
    }
  }

  static final _finalizer = NativeFinalizer(ffi.iox2lib
      .lookup<NativeFunction<Void Function(Pointer<Void>)>>('iox2_node_drop'));

  /// Get the node name
  String get name => _name;

  /// Check if the node is closed
  bool get isClosed => _isClosed;

  /// Create a publisher for the given service name
  Publisher publisher(String serviceName) {
    if (_isClosed) {
      throw const Iceoryx2Exception('Node is closed');
    }
    return Publisher._(this, serviceName);
  }

  /// Create a subscriber for the given service name
  Subscriber subscriber(String serviceName) {
    if (_isClosed) {
      throw const Iceoryx2Exception('Node is closed');
    }
    return Subscriber._(this, serviceName);
  }

  /// Close the node and release resources
  void close() {
    if (!_isClosed) {
      print('[Node] Closing node "$_name"');
      _finalizer.detach(this);
      ffi.iox2NodeDrop(_handle.cast());
      _isClosed = true;
      print('[Node] OK Node "$_name" closed');
    }
  }
}

/// Manages a publisher for sending messages
class Publisher implements Finalizable {
  late final Pointer<ffi.Iox2Publisher> _handle;
  final Node _node;
  final String _serviceName;
  bool _isClosed = false;

  /// Internal constructor - use Node.publisher() instead
  Publisher._(this._node, this._serviceName) {
    try {
      print('[Publisher] Creating publisher for service: "$_serviceName"');
      _handle = _createPublisher(_node._handle, _serviceName);
      _finalizer.attach(this, _handle.cast(), detach: this);
      print('[Publisher] OK Publisher created successfully');
    } catch (e) {
      print('[Publisher] ERROR Failed to create publisher: $e');
      rethrow;
    }
  }

  static final _finalizer = NativeFinalizer(ffi.iox2lib
      .lookup<NativeFunction<Void Function(Pointer<Void>)>>(
          'iox2_publisher_drop'));

  /// Get the service name
  String get serviceName => _serviceName;

  /// Check if the publisher is closed
  bool get isClosed => _isClosed;

  /// Send a message
  void send(Message message) {
    if (_isClosed) {
      throw const Iceoryx2Exception('Publisher is closed');
    }

    try {
      _publishMessage(_handle, message);
    } catch (e) {
      throw Iceoryx2Exception('Failed to send message: $e');
    }
  }

  /// Send a simple text message (convenience method)
  void sendText(String text, {String sender = 'unknown'}) {
    send(Message.create(text, sender: sender));
  }

  /// Close the publisher and release resources
  void close() {
    if (!_isClosed) {
      print('[Publisher] Closing publisher for service "$_serviceName"');
      _finalizer.detach(this);
      ffi.iox2PublisherDrop(_handle.cast());
      _isClosed = true;
      print('[Publisher] OK Publisher closed');
    }
  }

  // Private helper methods
  static Pointer<ffi.Iox2Publisher> _createPublisher(
      Pointer<ffi.Iox2Node> node, String serviceName) {
    final serviceNameUtf8 = serviceName.toNativeUtf8();
    final serviceNameHandlePtr = calloc<Pointer<Void>>();

    try {
      // Create service name
      final result = ffi.iox2ServiceNameNew(
          nullptr, serviceNameUtf8, serviceName.length, serviceNameHandlePtr);
      if (result != ffi.IOX2_OK) {
        throw Iceoryx2Exception('Failed to create service name', result);
      }

      final serviceNameHandle = serviceNameHandlePtr.value;
      final serviceNameCasted = ffi.iox2CastServiceNamePtr(serviceNameHandle);

      // Create service builder
      final nodeHandlePtr = calloc<Pointer<Void>>();
      nodeHandlePtr.value = node.cast();

      final serviceBuilder =
          ffi.iox2NodeServiceBuilder(nodeHandlePtr, nullptr, serviceNameCasted);
      calloc.free(nodeHandlePtr);

      if (serviceBuilder == nullptr) {
        throw const Iceoryx2Exception('Failed to create service builder');
      }

      // Transform to pub-sub service builder
      final pubSubServiceBuilder = ffi.iox2ServiceBuilderPubSub(serviceBuilder);
      if (pubSubServiceBuilder == nullptr) {
        throw const Iceoryx2Exception(
            'Failed to create pub-sub service builder');
      }

      // Set payload type details
      final payloadTypeName = "DartMessage".toNativeUtf8();
      final pubSubBuilderRef = calloc<Pointer<Void>>();
      pubSubBuilderRef.value = pubSubServiceBuilder;

      final payloadTypeResult =
          ffi.iox2ServiceBuilderPubSubSetPayloadTypeDetails(
              pubSubBuilderRef,
              ffi.IOX2_TYPE_VARIANT_FIXED_SIZE,
              payloadTypeName,
              "DartMessage".length,
              ffi.MESSAGE_STRUCT_SIZE,
              8 // 8-byte alignment
              );

      calloc.free(pubSubBuilderRef);
      calloc.free(payloadTypeName);

      if (payloadTypeResult != ffi.IOX2_OK) {
        throw Iceoryx2Exception(
            'Failed to set payload type details', payloadTypeResult);
      }

      // Open or create service
      final servicePtr = calloc<Pointer<Void>>();
      final serviceResult = ffi.iox2ServiceBuilderPubSubOpenOrCreate(
          pubSubServiceBuilder, nullptr, servicePtr);
      if (serviceResult != ffi.IOX2_OK) {
        calloc.free(servicePtr);
        throw Iceoryx2Exception(
            'Failed to open or create service', serviceResult);
      }

      final service = servicePtr.value;
      calloc.free(servicePtr);

      // Create publisher builder
      final serviceRef = calloc<Pointer<Void>>();
      serviceRef.value = service;
      final publisherBuilder =
          ffi.iox2PortFactoryPubSubPublisherBuilder(serviceRef, nullptr);
      calloc.free(serviceRef);

      if (publisherBuilder == nullptr) {
        throw const Iceoryx2Exception('Failed to create publisher builder');
      }

      // Create publisher
      final publisherPtr = calloc<Pointer<Void>>();
      final publisherResult = ffi.iox2PortFactoryPublisherBuilderCreate(
          publisherBuilder, nullptr, publisherPtr);
      if (publisherResult != ffi.IOX2_OK) {
        calloc.free(publisherPtr);
        throw Iceoryx2Exception('Failed to create publisher', publisherResult);
      }

      final publisher = publisherPtr.value;
      calloc.free(publisherPtr);
      return publisher.cast<ffi.Iox2Publisher>();
    } finally {
      calloc.free(serviceNameHandlePtr);
      calloc.free(serviceNameUtf8);
    }
  }

  static void _publishMessage(
      Pointer<ffi.Iox2Publisher> publisher, Message message) {
    final publisherRef = calloc<Pointer<Void>>();
    final samplePtr = calloc<Pointer<Void>>();
    final payloadPtr = calloc<Pointer<Void>>();

    publisherRef.value = publisher.cast();

    try {
      // Loan a sample
      final result =
          ffi.iox2PublisherLoanSliceUninit(publisherRef, nullptr, samplePtr, 1);
      if (result != ffi.IOX2_OK) {
        throw Iceoryx2Exception('Failed to loan sample', result);
      }

      final sample = samplePtr.value;

      // Get mutable payload from sample
      ffi.iox2SampleMutPayloadMut(samplePtr, payloadPtr, nullptr);
      final payload = payloadPtr.value;

      if (payload == nullptr) {
        throw const Iceoryx2Exception('Failed to get payload from sample');
      }

      // Serialize message
      final payloadData = payload.cast<Uint8>();
      MessageProtocol.serialize(message, payloadData);

      // Send the sample
      final sendResult = ffi.iox2SampleMutSend(sample, nullptr);
      if (sendResult != ffi.IOX2_OK) {
        throw Iceoryx2Exception('Failed to send message', sendResult);
      }
    } finally {
      calloc.free(publisherRef);
      calloc.free(samplePtr);
      calloc.free(payloadPtr);
    }
  }
}

/// Manages a subscriber for receiving messages
class Subscriber implements Finalizable {
  late final Pointer<ffi.Iox2Subscriber> _handle;
  final Node _node;
  final String _serviceName;
  bool _isClosed = false;

  // For event-driven messaging
  Isolate? _isolate;
  final ReceivePort _receivePort = ReceivePort();
  late final SendPort _sendPort;

  /// Stream of received messages
  Stream<Message> get messages => _receivePort.cast<Message>();

  /// Internal constructor - use Node.subscriber() instead
  Subscriber._(this._node, this._serviceName) {
    try {
      print('[Subscriber] Creating subscriber for service: "$_serviceName"');
      _handle = _createSubscriber(_node._handle, _serviceName);
      _sendPort = _receivePort.sendPort;
      _finalizer.attach(this, _handle.cast(), detach: this);

      // Don't start background listener automatically to avoid issues
      print('[Subscriber] OK Subscriber created successfully');
    } catch (e) {
      print('[Subscriber] ERROR Failed to create subscriber: $e');
      rethrow;
    }
  }

  static final _finalizer = NativeFinalizer(ffi.iox2lib
      .lookup<NativeFunction<Void Function(Pointer<Void>)>>(
          'iox2_subscriber_drop'));

  /// Get the service name
  String get serviceName => _serviceName;

  /// Check if the subscriber is closed
  bool get isClosed => _isClosed;

  /// Try to receive a message immediately (non-blocking)
  Message? tryReceive() {
    if (_isClosed) {
      throw const Iceoryx2Exception('Subscriber is closed');
    }

    try {
      return _receiveMessage(_handle);
    } catch (e) {
      print('[Subscriber] Error receiving message: $e');
      return null;
    }
  }

  /// Start event-driven message listening using waitset
  void startListening() {
    if (!_isClosed && _isolate == null) {
      _startListener();
    }
  }

  /// Close the subscriber and release resources
  void close() {
    if (!_isClosed) {
      print('[Subscriber] Closing subscriber for service "$_serviceName"');

      // Stop background isolate
      _isolate?.kill(priority: Isolate.immediate);
      _receivePort.close();

      _finalizer.detach(this);
      ffi.iox2SubscriberDrop(_handle.cast());
      _isClosed = true;
      print('[Subscriber] OK Subscriber closed');
    }
  }

  // Private helper methods
  void _startListener() async {
    _isolate = await Isolate.spawn(
      _listen,
      [_sendPort, _handle.address, _node._handle.address],
    );
  }

  static void _listen(List<dynamic> args) {
    final sendPort = args[0] as SendPort;
    final handleAddress = args[1] as int;
    final nodeAddress = args[2] as int;
    final handle = Pointer<ffi.Iox2Subscriber>.fromAddress(handleAddress);
    final nodeHandle = Pointer<ffi.Iox2Node>.fromAddress(nodeAddress);

    while (true) {
      try {
        // Try to receive available messages first
        final message = _receiveMessage(handle);
        if (message != null) {
          sendPort.send(message);
          continue; // Check for more messages immediately
        }

        // If no message available, use efficient waiting strategy
        // Instead of busy-waiting, use a moderate sleep with node wait fallback
        try {
          final nodeRef = calloc<Pointer<Void>>();
          nodeRef.value = nodeHandle.cast();

          // Try to wait for events with short timeout (100ms)
          final waitResult = ffi.iox2NodeWait(nodeRef, 100, 0);
          calloc.free(nodeRef);

          if (waitResult == ffi.IOX2_OK) {
            // Event detected, try to receive again immediately
            continue;
          }
        } catch (e) {
          // If waitset fails, fall back to sleep-based polling
          // This is safer than crashing
          sleep(const Duration(milliseconds: 50));
        }

        // Additional sleep to avoid excessive CPU usage
        sleep(const Duration(milliseconds: 50));
      } catch (e) {
        print('[Subscriber] Error in listener: $e');
        sleep(const Duration(milliseconds: 100));
      }
    }
  }

  static Pointer<ffi.Iox2Subscriber> _createSubscriber(
      Pointer<ffi.Iox2Node> node, String serviceName) {
    final serviceNameUtf8 = serviceName.toNativeUtf8();
    final serviceNameHandlePtr = calloc<Pointer<Void>>();

    try {
      // Create service name
      final result = ffi.iox2ServiceNameNew(
          nullptr, serviceNameUtf8, serviceName.length, serviceNameHandlePtr);
      if (result != ffi.IOX2_OK) {
        throw Iceoryx2Exception('Failed to create service name', result);
      }

      final serviceNameHandle = serviceNameHandlePtr.value;
      final serviceNameCasted = ffi.iox2CastServiceNamePtr(serviceNameHandle);

      // Create service builder
      final nodeHandlePtr = calloc<Pointer<Void>>();
      nodeHandlePtr.value = node.cast();

      final serviceBuilder =
          ffi.iox2NodeServiceBuilder(nodeHandlePtr, nullptr, serviceNameCasted);
      calloc.free(nodeHandlePtr);

      if (serviceBuilder == nullptr) {
        throw const Iceoryx2Exception('Failed to create service builder');
      }

      // Transform to pub-sub service builder
      final pubSubServiceBuilder = ffi.iox2ServiceBuilderPubSub(serviceBuilder);
      if (pubSubServiceBuilder == nullptr) {
        throw const Iceoryx2Exception(
            'Failed to create pub-sub service builder');
      }

      // Set payload type details
      final payloadTypeName = "DartMessage".toNativeUtf8();
      final pubSubBuilderRef = calloc<Pointer<Void>>();
      pubSubBuilderRef.value = pubSubServiceBuilder;

      final payloadTypeResult =
          ffi.iox2ServiceBuilderPubSubSetPayloadTypeDetails(
              pubSubBuilderRef,
              ffi.IOX2_TYPE_VARIANT_FIXED_SIZE,
              payloadTypeName,
              "DartMessage".length,
              ffi.MESSAGE_STRUCT_SIZE,
              8 // 8-byte alignment
              );

      calloc.free(pubSubBuilderRef);
      calloc.free(payloadTypeName);

      if (payloadTypeResult != ffi.IOX2_OK) {
        throw Iceoryx2Exception(
            'Failed to set payload type details', payloadTypeResult);
      }

      // Open or create service
      final servicePtr = calloc<Pointer<Void>>();
      final serviceResult = ffi.iox2ServiceBuilderPubSubOpenOrCreate(
          pubSubServiceBuilder, nullptr, servicePtr);
      if (serviceResult != ffi.IOX2_OK) {
        calloc.free(servicePtr);
        throw Iceoryx2Exception(
            'Failed to open or create service', serviceResult);
      }

      final service = servicePtr.value;
      calloc.free(servicePtr);

      // Create subscriber builder
      final serviceRef = calloc<Pointer<Void>>();
      serviceRef.value = service;
      final subscriberBuilder =
          ffi.iox2PortFactoryPubSubSubscriberBuilder(serviceRef, nullptr);
      calloc.free(serviceRef);

      if (subscriberBuilder == nullptr) {
        throw const Iceoryx2Exception('Failed to create subscriber builder');
      }

      // Create subscriber
      final subscriberPtr = calloc<Pointer<Void>>();
      final subscriberResult = ffi.iox2PortFactorySubscriberBuilderCreate(
          subscriberBuilder, nullptr, subscriberPtr);
      if (subscriberResult != ffi.IOX2_OK) {
        calloc.free(subscriberPtr);
        throw Iceoryx2Exception(
            'Failed to create subscriber', subscriberResult);
      }

      final subscriber = subscriberPtr.value;
      calloc.free(subscriberPtr);
      return subscriber.cast<ffi.Iox2Subscriber>();
    } finally {
      calloc.free(serviceNameHandlePtr);
      calloc.free(serviceNameUtf8);
    }
  }

  static Message? _receiveMessage(Pointer<ffi.Iox2Subscriber> subscriber) {
    final subscriberRef = calloc<Pointer<Void>>();
    final samplePtr = calloc<Pointer<Void>>();

    subscriberRef.value = subscriber.cast();

    try {
      final result =
          ffi.iox2SubscriberReceive(subscriberRef, nullptr, samplePtr);
      if (result != ffi.IOX2_OK) {
        return null; // No data available or error
      }

      final sample = samplePtr.value;
      if (sample == nullptr) {
        return null; // No sample available
      }

      // Get payload from sample
      final payloadPtr = calloc<Pointer<Void>>();
      ffi.iox2SamplePayload(samplePtr, payloadPtr, nullptr);
      final payload = payloadPtr.value;

      if (payload == nullptr) {
        calloc.free(payloadPtr);
        ffi.iox2SampleDrop(sample);
        return null;
      }

      try {
        // Deserialize message
        final payloadData = payload.cast<Uint8>();
        final message = MessageProtocol.deserialize(payloadData);

        // Clean up resources
        calloc.free(payloadPtr);
        ffi.iox2SampleDrop(sample);
        return message;
      } catch (e) {
        print('[Subscriber] Error deserializing message: $e');
        calloc.free(payloadPtr);
        ffi.iox2SampleDrop(sample);
        return null;
      }
    } finally {
      calloc.free(subscriberRef);
      calloc.free(samplePtr);
    }
  }
}
