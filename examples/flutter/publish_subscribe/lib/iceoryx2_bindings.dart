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

import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';

// Load the iceoryx2 FFI library
final DynamicLibrary _lib = Platform.isLinux
    ? DynamicLibrary.open(
        '/home/youndong/projects/iceoryx2/iceoryx2/target/release/libiceoryx2_ffi.so')
    : throw UnsupportedError('Platform not supported');

// Opaque pointer types for iceoryx2 objects
final class Iox2Node extends Opaque {}

final class Iox2NodeBuilder extends Opaque {}

final class Iox2Publisher extends Opaque {}

final class Iox2Subscriber extends Opaque {}

final class Iox2Sample extends Opaque {}

final class Iox2PortFactory extends Opaque {}

final class Iox2WaitSet extends Opaque {}

// Result enum values
const int IOX2_OK = 0;
const int IOX2_ERROR = 1;

// Service type enum values
const int IOX2_SERVICE_TYPE_LOCAL = 0;
const int IOX2_SERVICE_TYPE_IPC = 1;

// Type variant enum values
const int IOX2_TYPE_VARIANT_FIXED_SIZE = 0;
const int IOX2_TYPE_VARIANT_DYNAMIC = 1;

// Message structure constants
const int MESSAGE_MAX_LENGTH = 256;
const int MESSAGE_STRUCT_SIZE = 264; // 256 bytes message + 8 bytes length field

// Type definitions matching the C API
typedef Iox2NodePtr = Pointer<Iox2Node>;
typedef Iox2NodeBuilderPtr = Pointer<Iox2NodeBuilder>;
typedef Iox2PublisherPtr = Pointer<Iox2Publisher>;
typedef Iox2SubscriberPtr = Pointer<Iox2Subscriber>;
typedef Iox2SamplePtr = Pointer<Iox2Sample>;
typedef Iox2PortFactoryPtr = Pointer<Iox2PortFactory>;

// Node wait functions
typedef _IoxNodeWaitC = Int32 Function(Pointer<Pointer<Void>>, Uint64, Uint64);
typedef _IoxNodeWait = int Function(Pointer<Pointer<Void>>, int, int);

typedef _IoxSubscriberReceiveC = Int32 Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>);
typedef _IoxSubscriberReceive = int Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>);

// Function signatures
typedef _IoxNodeBuilderNewC = Pointer<Void> Function(Pointer<Void>);
typedef _IoxNodeBuilderNew = Pointer<Void> Function(Pointer<Void>);

typedef _IoxNodeBuilderCreateC = Int32 Function(
    Pointer<Void>, Pointer<Void>, Int32, Pointer<Pointer<Void>>);
typedef _IoxNodeBuilderCreate = int Function(
    Pointer<Void>, Pointer<Void>, int, Pointer<Pointer<Void>>);

typedef _IoxNodeServiceBuilderC = Pointer<Void> Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Void>);
typedef _IoxNodeServiceBuilder = Pointer<Void> Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Void>);

// Service name functions
typedef _IoxServiceNameNewC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Size, Pointer<Pointer<Void>>);
typedef _IoxServiceNameNew = int Function(
    Pointer<Void>, Pointer<Utf8>, int, Pointer<Pointer<Void>>);

typedef _IoxCastServiceNamePtrC = Pointer<Void> Function(Pointer<Void>);
typedef _IoxCastServiceNamePtr = Pointer<Void> Function(Pointer<Void>);

typedef _IoxServiceBuilderPubSubC = Pointer<Void> Function(Pointer<Void>);
typedef _IoxServiceBuilderPubSub = Pointer<Void> Function(Pointer<Void>);

typedef _IoxServiceBuilderPubSubSetPayloadTypeDetailsC = Int32 Function(
    Pointer<Pointer<Void>>, Int32, Pointer<Utf8>, Size, Size, Size);
typedef _IoxServiceBuilderPubSubSetPayloadTypeDetails = int Function(
    Pointer<Pointer<Void>>, int, Pointer<Utf8>, int, int, int);

typedef _IoxServiceBuilderPubSubOpenOrCreateC = Int32 Function(
    Pointer<Void>, Pointer<Void>, Pointer<Pointer<Void>>);
typedef _IoxServiceBuilderPubSubOpenOrCreate = int Function(
    Pointer<Void>, Pointer<Void>, Pointer<Pointer<Void>>);

typedef _IoxPortFactoryPubSubPublisherBuilderC = Pointer<Void> Function(
    Pointer<Pointer<Void>>, Pointer<Void>);
typedef _IoxPortFactoryPubSubPublisherBuilder = Pointer<Void> Function(
    Pointer<Pointer<Void>>, Pointer<Void>);

typedef _IoxPortFactoryPublisherBuilderCreateC = Int32 Function(
    Pointer<Void>, Pointer<Void>, Pointer<Pointer<Void>>);
typedef _IoxPortFactoryPublisherBuilderCreate = int Function(
    Pointer<Void>, Pointer<Void>, Pointer<Pointer<Void>>);

typedef _IoxPortFactoryPubSubSubscriberBuilderC = Pointer<Void> Function(
    Pointer<Pointer<Void>>, Pointer<Void>);
typedef _IoxPortFactoryPubSubSubscriberBuilder = Pointer<Void> Function(
    Pointer<Pointer<Void>>, Pointer<Void>);

typedef _IoxPortFactorySubscriberBuilderCreateC = Int32 Function(
    Pointer<Void>, Pointer<Void>, Pointer<Pointer<Void>>);
typedef _IoxPortFactorySubscriberBuilderCreate = int Function(
    Pointer<Void>, Pointer<Void>, Pointer<Pointer<Void>>);

typedef _IoxPublisherLoanSliceUninitC = Int32 Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>, Size);
typedef _IoxPublisherLoanSliceUninit = int Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>, int);

typedef _IoxSampleMutPayloadMutC = Void Function(
    Pointer<Pointer<Void>>, Pointer<Pointer<Void>>, Pointer<Void>);
typedef _IoxSampleMutPayloadMut = void Function(
    Pointer<Pointer<Void>>, Pointer<Pointer<Void>>, Pointer<Void>);

typedef _IoxSampleMutSendC = Int32 Function(Pointer<Void>, Pointer<Void>);
typedef _IoxSampleMutSend = int Function(Pointer<Void>, Pointer<Void>);

typedef _IoxSamplePayloadC = Void Function(
    Pointer<Pointer<Void>>, Pointer<Pointer<Void>>, Pointer<Void>);
typedef _IoxSamplePayload = void Function(
    Pointer<Pointer<Void>>, Pointer<Pointer<Void>>, Pointer<Void>);

typedef _IoxNodeDropC = Void Function(Pointer<Void>);
typedef _IoxNodeDrop = void Function(Pointer<Void>);

typedef _IoxPublisherDropC = Void Function(Pointer<Void>);
typedef _IoxPublisherDrop = void Function(Pointer<Void>);

typedef _IoxSubscriberDropC = Void Function(Pointer<Void>);
typedef _IoxSubscriberDrop = void Function(Pointer<Void>);

typedef _IoxSampleDropC = Void Function(Pointer<Void>);
typedef _IoxSampleDrop = void Function(Pointer<Void>);

// Simple message helper functions
class MessageHelper {
  // Write a string message to memory with length prefix
  static void writeMessage(Pointer<Uint8> memory, String message) {
    // Clear first 264 bytes to ensure clean memory
    for (int i = 0; i < MESSAGE_STRUCT_SIZE; i++) {
      memory[i] = 0;
    }

    final messageBytes = message.codeUnits;
    final actualLength = messageBytes.length > MESSAGE_MAX_LENGTH
        ? MESSAGE_MAX_LENGTH
        : messageBytes.length;

    // Write length as first 8 bytes (Uint64)
    final lengthPtr = Pointer<Uint64>.fromAddress(memory.address);
    lengthPtr.value = actualLength;

    // Write message data starting from offset 8
    final dataPtr = memory.elementAt(8);
    for (int i = 0; i < actualLength; i++) {
      dataPtr[i] = messageBytes[i];
    }
  }

  // Read a string message from memory with length prefix
  static String readMessage(Pointer<Uint8> memory) {
    // Read length from first 8 bytes
    final lengthPtr = Pointer<Uint64>.fromAddress(memory.address);
    final length = lengthPtr.value;

    // Validate length
    if (length > MESSAGE_MAX_LENGTH) {
      throw Exception('Invalid message length: $length');
    }

    // Read message data starting from offset 8
    final dataPtr = memory.elementAt(8);
    final messageBytes = <int>[];

    for (int i = 0; i < length; i++) {
      messageBytes.add(dataPtr[i]);
    }

    return String.fromCharCodes(messageBytes);
  }
}

// Dart wrapper class for iceoryx2 functionality
class Iceoryx2 {
  // Node wait and subscriber receive FFI bindings
  static final _iox2NodeWait = _lib
      .lookup<NativeFunction<_IoxNodeWaitC>>('iox2_node_wait')
      .asFunction<_IoxNodeWait>();

  static final _iox2SubscriberReceive = _lib
      .lookup<NativeFunction<_IoxSubscriberReceiveC>>('iox2_subscriber_receive')
      .asFunction<_IoxSubscriberReceive>();

  // Event-driven node wait method
  static int nodeWait(Pointer<Void> node,
      {int timeoutSecs = 1, int timeoutNsecs = 0}) {
    final nodeRef = calloc<Pointer<Void>>();
    nodeRef.value = node;
    try {
      return _iox2NodeWait(nodeRef, timeoutSecs, timeoutNsecs);
    } finally {
      calloc.free(nodeRef);
    }
  }

  // Function bindings
  static final _iox2NodeBuilderNew = _lib
      .lookup<NativeFunction<_IoxNodeBuilderNewC>>('iox2_node_builder_new')
      .asFunction<_IoxNodeBuilderNew>();

  static final _iox2NodeBuilderCreate = _lib
      .lookup<NativeFunction<_IoxNodeBuilderCreateC>>(
          'iox2_node_builder_create')
      .asFunction<_IoxNodeBuilderCreate>();

  static final _iox2NodeServiceBuilder = _lib
      .lookup<NativeFunction<_IoxNodeServiceBuilderC>>(
          'iox2_node_service_builder')
      .asFunction<_IoxNodeServiceBuilder>();

  static final iox2ServiceNameNew = _lib
      .lookup<NativeFunction<_IoxServiceNameNewC>>('iox2_service_name_new')
      .asFunction<_IoxServiceNameNew>();

  static final iox2CastServiceNamePtr = _lib
      .lookup<NativeFunction<_IoxCastServiceNamePtrC>>(
          'iox2_cast_service_name_ptr')
      .asFunction<_IoxCastServiceNamePtr>();

  static final iox2ServiceBuilderPubSub = _lib
      .lookup<NativeFunction<_IoxServiceBuilderPubSubC>>(
          'iox2_service_builder_pub_sub')
      .asFunction<_IoxServiceBuilderPubSub>();

  static final iox2ServiceBuilderPubSubSetPayloadTypeDetails = _lib
      .lookup<NativeFunction<_IoxServiceBuilderPubSubSetPayloadTypeDetailsC>>(
          'iox2_service_builder_pub_sub_set_payload_type_details')
      .asFunction<_IoxServiceBuilderPubSubSetPayloadTypeDetails>();

  static final iox2ServiceBuilderPubSubOpenOrCreate = _lib
      .lookup<NativeFunction<_IoxServiceBuilderPubSubOpenOrCreateC>>(
          'iox2_service_builder_pub_sub_open_or_create')
      .asFunction<_IoxServiceBuilderPubSubOpenOrCreate>();

  static final iox2PortFactoryPubSubPublisherBuilder = _lib
      .lookup<NativeFunction<_IoxPortFactoryPubSubPublisherBuilderC>>(
          'iox2_port_factory_pub_sub_publisher_builder')
      .asFunction<_IoxPortFactoryPubSubPublisherBuilder>();

  static final _iox2PortFactoryPublisherBuilderCreate = _lib
      .lookup<NativeFunction<_IoxPortFactoryPublisherBuilderCreateC>>(
          'iox2_port_factory_publisher_builder_create')
      .asFunction<_IoxPortFactoryPublisherBuilderCreate>();

  static final iox2PortFactoryPubSubSubscriberBuilder = _lib
      .lookup<NativeFunction<_IoxPortFactoryPubSubSubscriberBuilderC>>(
          'iox2_port_factory_pub_sub_subscriber_builder')
      .asFunction<_IoxPortFactoryPubSubSubscriberBuilder>();

  static final _iox2PortFactorySubscriberBuilderCreate = _lib
      .lookup<NativeFunction<_IoxPortFactorySubscriberBuilderCreateC>>(
          'iox2_port_factory_subscriber_builder_create')
      .asFunction<_IoxPortFactorySubscriberBuilderCreate>();

  static final _iox2PublisherLoanSliceUninit = _lib
      .lookup<NativeFunction<_IoxPublisherLoanSliceUninitC>>(
          'iox2_publisher_loan_slice_uninit')
      .asFunction<_IoxPublisherLoanSliceUninit>();

  static final _iox2SampleMutPayloadMut = _lib
      .lookup<NativeFunction<_IoxSampleMutPayloadMutC>>(
          'iox2_sample_mut_payload_mut')
      .asFunction<_IoxSampleMutPayloadMut>();

  static final _iox2SampleMutSend = _lib
      .lookup<NativeFunction<_IoxSampleMutSendC>>('iox2_sample_mut_send')
      .asFunction<_IoxSampleMutSend>();

  static final _iox2SamplePayload = _lib
      .lookup<NativeFunction<_IoxSamplePayloadC>>('iox2_sample_payload')
      .asFunction<_IoxSamplePayload>();

  static final _iox2NodeDrop = _lib
      .lookup<NativeFunction<_IoxNodeDropC>>('iox2_node_drop')
      .asFunction<_IoxNodeDrop>();

  static final _iox2PublisherDrop = _lib
      .lookup<NativeFunction<_IoxPublisherDropC>>('iox2_publisher_drop')
      .asFunction<_IoxPublisherDrop>();

  static final _iox2SubscriberDrop = _lib
      .lookup<NativeFunction<_IoxSubscriberDropC>>('iox2_subscriber_drop')
      .asFunction<_IoxSubscriberDrop>();

  static final _iox2SampleDrop = _lib
      .lookup<NativeFunction<_IoxSampleDropC>>('iox2_sample_drop')
      .asFunction<_IoxSampleDrop>();

  // High-level API methods
  static Pointer<Void> createNode() {
    print('[FFI] Creating node builder...');
    final nodeBuilder = _iox2NodeBuilderNew(nullptr);
    if (nodeBuilder == nullptr) {
      throw Exception('Failed to create node builder');
    }

    print('[FFI] Building node...');
    final nodePtr = calloc<Pointer<Void>>();
    try {
      final result = _iox2NodeBuilderCreate(
          nodeBuilder, nullptr, IOX2_SERVICE_TYPE_IPC, nodePtr);
      if (result != IOX2_OK) {
        throw Exception('Failed to create node: $result');
      }
      print('[FFI] Node created successfully');
      return nodePtr.value;
    } finally {
      calloc.free(nodePtr);
    }
  }

  static Pointer<Void> createPublisher(Pointer<Void> node, String serviceName) {
    print('[FFI] Creating publisher for service: "$serviceName"');

    // 1. Create service name
    final serviceNamePtr = serviceName.toNativeUtf8();
    final serviceNameHandlePtr = calloc<Pointer<Void>>();

    try {
      final result = iox2ServiceNameNew(
          nullptr, serviceNamePtr, serviceName.length, serviceNameHandlePtr);
      if (result != IOX2_OK) {
        throw Exception('Failed to create service name: $result');
      }

      final serviceNameHandle = serviceNameHandlePtr.value;

      // 2. Cast service name to pointer
      final serviceNameCastedPtr = iox2CastServiceNamePtr(serviceNameHandle);

      // 3. Create service builder (pass nullptr to let it allocate)
      final nodeHandlePtr = calloc<Pointer<Void>>();
      nodeHandlePtr.value = node;

      final serviceBuilder =
          _iox2NodeServiceBuilder(nodeHandlePtr, nullptr, serviceNameCastedPtr);
      if (serviceBuilder == nullptr) {
        calloc.free(nodeHandlePtr);
        throw Exception('Failed to create service builder');
      }

      calloc.free(nodeHandlePtr);

      // 4. Transform to pub-sub service builder
      final pubSubServiceBuilder = iox2ServiceBuilderPubSub(serviceBuilder);
      if (pubSubServiceBuilder == nullptr) {
        throw Exception('Failed to create pub-sub service builder');
      }

      // 5. Set payload type details for our message structure
      final payloadTypeName = "DartMessage".toNativeUtf8();
      final pubSubBuilderRef = calloc<Pointer<Void>>();
      pubSubBuilderRef.value = pubSubServiceBuilder;

      final payloadTypeResult = iox2ServiceBuilderPubSubSetPayloadTypeDetails(
          pubSubBuilderRef,
          IOX2_TYPE_VARIANT_FIXED_SIZE,
          payloadTypeName,
          "DartMessage".length,
          MESSAGE_STRUCT_SIZE,
          8 // 8-byte alignment for our message structure
          );

      calloc.free(pubSubBuilderRef);
      calloc.free(payloadTypeName);

      if (payloadTypeResult != IOX2_OK) {
        throw Exception(
            'Failed to set payload type details: $payloadTypeResult');
      }

      print(
          '[FFI] Set payload type: DartMessage, size=$MESSAGE_STRUCT_SIZE, alignment=8');

      // 6. Open or create service
      final servicePtr = calloc<Pointer<Void>>();
      final serviceResult = iox2ServiceBuilderPubSubOpenOrCreate(
          pubSubServiceBuilder, nullptr, servicePtr);
      if (serviceResult != IOX2_OK) {
        calloc.free(servicePtr);
        throw Exception('Failed to open or create service: $serviceResult');
      }

      final service = servicePtr.value;
      calloc.free(servicePtr);

      // 7. Create publisher builder
      final serviceRef = calloc<Pointer<Void>>();
      serviceRef.value = service;
      final publisherBuilder =
          iox2PortFactoryPubSubPublisherBuilder(serviceRef, nullptr);
      calloc.free(serviceRef);

      if (publisherBuilder == nullptr) {
        throw Exception('Failed to create publisher builder');
      }

      // 8. Create publisher
      final publisherPtr = calloc<Pointer<Void>>();
      final publisherResult = _iox2PortFactoryPublisherBuilderCreate(
          publisherBuilder, nullptr, publisherPtr);
      if (publisherResult != IOX2_OK) {
        calloc.free(publisherPtr);
        throw Exception('Failed to create publisher: $publisherResult');
      }

      print('[FFI] Publisher created successfully');
      return publisherPtr.value;
    } finally {
      calloc.free(serviceNameHandlePtr);
      calloc.free(serviceNamePtr);
    }
  }

  static Pointer<Void> createSubscriber(
      Pointer<Void> node, String serviceName) {
    print('[FFI] Creating subscriber for service: "$serviceName"');

    // 1. Create service name
    final serviceNamePtr = serviceName.toNativeUtf8();
    final serviceNameHandlePtr = calloc<Pointer<Void>>();

    try {
      final result = iox2ServiceNameNew(
          nullptr, serviceNamePtr, serviceName.length, serviceNameHandlePtr);
      if (result != IOX2_OK) {
        throw Exception('Failed to create service name: $result');
      }

      final serviceNameHandle = serviceNameHandlePtr.value;

      // 2. Cast service name to pointer
      final serviceNameCastedPtr = iox2CastServiceNamePtr(serviceNameHandle);

      // 3. Create service builder (pass nullptr to let it allocate)
      final nodeHandlePtr = calloc<Pointer<Void>>();
      nodeHandlePtr.value = node;

      final serviceBuilder =
          _iox2NodeServiceBuilder(nodeHandlePtr, nullptr, serviceNameCastedPtr);
      if (serviceBuilder == nullptr) {
        calloc.free(nodeHandlePtr);
        throw Exception('Failed to create service builder');
      }

      calloc.free(nodeHandlePtr);

      // 4. Transform to pub-sub service builder
      final pubSubServiceBuilder = iox2ServiceBuilderPubSub(serviceBuilder);
      if (pubSubServiceBuilder == nullptr) {
        throw Exception('Failed to create pub-sub service builder');
      }

      // 5. Set payload type details for our message structure
      final payloadTypeName = "DartMessage".toNativeUtf8();
      final pubSubBuilderRef = calloc<Pointer<Void>>();
      pubSubBuilderRef.value = pubSubServiceBuilder;

      final payloadTypeResult = iox2ServiceBuilderPubSubSetPayloadTypeDetails(
          pubSubBuilderRef,
          IOX2_TYPE_VARIANT_FIXED_SIZE,
          payloadTypeName,
          "DartMessage".length,
          MESSAGE_STRUCT_SIZE,
          8 // 8-byte alignment for our message structure
          );

      calloc.free(pubSubBuilderRef);
      calloc.free(payloadTypeName);

      if (payloadTypeResult != IOX2_OK) {
        throw Exception(
            'Failed to set payload type details: $payloadTypeResult');
      }

      print(
          '[FFI] Set payload type: DartMessage, size=$MESSAGE_STRUCT_SIZE, alignment=8');

      // 6. Open or create service
      final servicePtr = calloc<Pointer<Void>>();
      final serviceResult = iox2ServiceBuilderPubSubOpenOrCreate(
          pubSubServiceBuilder, nullptr, servicePtr);
      if (serviceResult != IOX2_OK) {
        calloc.free(servicePtr);
        throw Exception('Failed to open or create service: $serviceResult');
      }

      final service = servicePtr.value;
      calloc.free(servicePtr);

      // 7. Create subscriber builder
      final serviceRef = calloc<Pointer<Void>>();
      serviceRef.value = service;
      final subscriberBuilder =
          iox2PortFactoryPubSubSubscriberBuilder(serviceRef, nullptr);
      calloc.free(serviceRef);

      if (subscriberBuilder == nullptr) {
        throw Exception('Failed to create subscriber builder');
      }

      // 8. Create subscriber
      final subscriberPtr = calloc<Pointer<Void>>();
      final subscriberResult = _iox2PortFactorySubscriberBuilderCreate(
          subscriberBuilder, nullptr, subscriberPtr);
      if (subscriberResult != IOX2_OK) {
        calloc.free(subscriberPtr);
        throw Exception('Failed to create subscriber: $subscriberResult');
      }

      print('[FFI] Subscriber created successfully');
      return subscriberPtr.value;
    } finally {
      calloc.free(serviceNameHandlePtr);
      calloc.free(serviceNamePtr);
    }
  }

  static void publish(Pointer<Void> publisher, String message) {
    final publisherRef = calloc<Pointer<Void>>();
    final samplePtr = calloc<Pointer<Void>>();
    final payloadPtr = calloc<Pointer<Void>>();

    publisherRef.value = publisher;

    try {
      print('[FFI] Publishing message: "$message" (${message.length} chars)');

      // Loan a single element (our DartMessage structure)
      final result =
          _iox2PublisherLoanSliceUninit(publisherRef, nullptr, samplePtr, 1);
      if (result != IOX2_OK) {
        throw Exception('Failed to loan sample: $result');
      }

      final sample = samplePtr.value;

      // Get mutable payload from sample
      _iox2SampleMutPayloadMut(samplePtr, payloadPtr, nullptr);
      final payload = payloadPtr.value;

      if (payload == nullptr) {
        throw Exception('Failed to get payload from sample');
      }

      // Write message using our structured format
      final payloadData = payload.cast<Uint8>();
      MessageHelper.writeMessage(payloadData, message);

      print('[FFI] Message written to DartMessage structure');

      // Send the sample
      final sendResult = _iox2SampleMutSend(sample, nullptr);
      if (sendResult != IOX2_OK) {
        throw Exception('Failed to send message: $sendResult');
      }

      print('[FFI] Message sent successfully');
    } finally {
      calloc.free(publisherRef);
      calloc.free(samplePtr);
      calloc.free(payloadPtr);
    }
  }

  static String? receive(Pointer<Void> subscriber) {
    final subscriberRef = calloc<Pointer<Void>>();
    final samplePtr = calloc<Pointer<Void>>();

    subscriberRef.value = subscriber;

    try {
      final result = _iox2SubscriberReceive(subscriberRef, nullptr, samplePtr);
      if (result != IOX2_OK) {
        return null; // No data available or error
      }

      final sample = samplePtr.value;
      if (sample == nullptr) {
        return null; // No sample available
      }

      // Get payload from sample using the correct API
      final payloadPtr = calloc<Pointer<Void>>();
      _iox2SamplePayload(samplePtr, payloadPtr, nullptr);
      final payload = payloadPtr.value;

      if (payload == nullptr) {
        calloc.free(payloadPtr);
        _iox2SampleDrop(sample);
        return null;
      }

      try {
        // Read message using our structured format
        final payloadData = payload.cast<Uint8>();
        final message = MessageHelper.readMessage(payloadData);

        print('[FFI] Successfully decoded DartMessage: "$message"');

        // Clean up resources
        calloc.free(payloadPtr);
        _iox2SampleDrop(sample);
        return message;
      } catch (e) {
        print('[FFI] Error reading DartMessage structure: $e');
        calloc.free(payloadPtr);
        _iox2SampleDrop(sample);
        return null;
      }
    } finally {
      calloc.free(subscriberRef);
      calloc.free(samplePtr);
    }
  }

  static void cleanup(Pointer<Void> node,
      {Pointer<Void>? publisher, Pointer<Void>? subscriber}) {
    if (publisher != null) {
      _iox2PublisherDrop(publisher);
    }
    if (subscriber != null) {
      _iox2SubscriberDrop(subscriber);
    }
    _iox2NodeDrop(node);
  }

  // Individual drop methods for cleanup
  static void dropPublisher(Pointer<Void> publisher) {
    _iox2PublisherDrop(publisher);
  }

  static void dropSubscriber(Pointer<Void> subscriber) {
    _iox2SubscriberDrop(subscriber);
  }

  static void dropNode(Pointer<Void> node) {
    _iox2NodeDrop(node);
  }
}
