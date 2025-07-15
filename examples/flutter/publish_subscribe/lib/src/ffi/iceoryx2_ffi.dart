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

/// Pure FFI bindings for iceoryx2
/// This file contains only pure FFI declarations including C function signatures, typedefs, and lookups.
library iceoryx2_ffi;

import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';

// Load the iceoryx2 FFI library
final DynamicLibrary iox2lib = Platform.isLinux
    ? DynamicLibrary.open(
        '/home/youndong/projects/iceoryx2/iceoryx2/target/release/libiceoryx2_ffi.so')
    : throw UnsupportedError('Platform not supported');

// =============================================================================
// OPAQUE TYPES
// =============================================================================

/// Opaque pointer types for iceoryx2 objects
final class Iox2Node extends Opaque {}

final class Iox2NodeBuilder extends Opaque {}

final class Iox2Publisher extends Opaque {}

final class Iox2Subscriber extends Opaque {}

final class Iox2Sample extends Opaque {}

final class Iox2PortFactory extends Opaque {}

final class Iox2WaitSet extends Opaque {}

// =============================================================================
// CONSTANTS
// =============================================================================

/// Result enum values
const int IOX2_OK = 0;
const int IOX2_ERROR = 1;

/// Service type enum values
const int IOX2_SERVICE_TYPE_LOCAL = 0;
const int IOX2_SERVICE_TYPE_IPC = 1;

/// Type variant enum values
const int IOX2_TYPE_VARIANT_FIXED_SIZE = 0;
const int IOX2_TYPE_VARIANT_DYNAMIC = 1;

/// Message structure constants
const int MESSAGE_MAX_LENGTH = 256;
const int MESSAGE_STRUCT_SIZE = 264; // 256 bytes message + 8 bytes length field

// =============================================================================
// TYPE ALIASES
// =============================================================================

typedef Iox2NodePtr = Pointer<Iox2Node>;
typedef Iox2NodeBuilderPtr = Pointer<Iox2NodeBuilder>;
typedef Iox2PublisherPtr = Pointer<Iox2Publisher>;
typedef Iox2SubscriberPtr = Pointer<Iox2Subscriber>;
typedef Iox2SamplePtr = Pointer<Iox2Sample>;
typedef Iox2PortFactoryPtr = Pointer<Iox2PortFactory>;

// =============================================================================
// C FUNCTION SIGNATURES (typedefs)
// =============================================================================

// Node functions
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

typedef _IoxNodeWaitC = Int32 Function(Pointer<Pointer<Void>>, Uint64, Uint64);
typedef _IoxNodeWait = int Function(Pointer<Pointer<Void>>, int, int);

typedef _IoxNodeDropC = Void Function(Pointer<Void>);
typedef _IoxNodeDrop = void Function(Pointer<Void>);

// Service name functions
typedef _IoxServiceNameNewC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Size, Pointer<Pointer<Void>>);
typedef _IoxServiceNameNew = int Function(
    Pointer<Void>, Pointer<Utf8>, int, Pointer<Pointer<Void>>);

typedef _IoxCastServiceNamePtrC = Pointer<Void> Function(Pointer<Void>);
typedef _IoxCastServiceNamePtr = Pointer<Void> Function(Pointer<Void>);

// Service builder functions
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

// Port factory functions
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

// Publisher functions
typedef _IoxPublisherLoanSliceUninitC = Int32 Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>, Size);
typedef _IoxPublisherLoanSliceUninit = int Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>, int);

typedef _IoxPublisherDropC = Void Function(Pointer<Void>);
typedef _IoxPublisherDrop = void Function(Pointer<Void>);

// Subscriber functions
typedef _IoxSubscriberReceiveC = Int32 Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>);
typedef _IoxSubscriberReceive = int Function(
    Pointer<Pointer<Void>>, Pointer<Void>, Pointer<Pointer<Void>>);

typedef _IoxSubscriberDropC = Void Function(Pointer<Void>);
typedef _IoxSubscriberDrop = void Function(Pointer<Void>);

// Sample functions
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

typedef _IoxSampleDropC = Void Function(Pointer<Void>);
typedef _IoxSampleDrop = void Function(Pointer<Void>);

// =============================================================================
// FFI FUNCTION LOOKUPS
// =============================================================================

// Node functions
final iox2NodeBuilderNew = iox2lib
    .lookup<NativeFunction<_IoxNodeBuilderNewC>>('iox2_node_builder_new')
    .asFunction<_IoxNodeBuilderNew>();

final iox2NodeBuilderCreate = iox2lib
    .lookup<NativeFunction<_IoxNodeBuilderCreateC>>('iox2_node_builder_create')
    .asFunction<_IoxNodeBuilderCreate>();

final iox2NodeServiceBuilder = iox2lib
    .lookup<NativeFunction<_IoxNodeServiceBuilderC>>(
        'iox2_node_service_builder')
    .asFunction<_IoxNodeServiceBuilder>();

final iox2NodeWait = iox2lib
    .lookup<NativeFunction<_IoxNodeWaitC>>('iox2_node_wait')
    .asFunction<_IoxNodeWait>();

final iox2NodeDrop = iox2lib
    .lookup<NativeFunction<_IoxNodeDropC>>('iox2_node_drop')
    .asFunction<_IoxNodeDrop>();

// Service name functions
final iox2ServiceNameNew = iox2lib
    .lookup<NativeFunction<_IoxServiceNameNewC>>('iox2_service_name_new')
    .asFunction<_IoxServiceNameNew>();

final iox2CastServiceNamePtr = iox2lib
    .lookup<NativeFunction<_IoxCastServiceNamePtrC>>(
        'iox2_cast_service_name_ptr')
    .asFunction<_IoxCastServiceNamePtr>();

// Service builder functions
final iox2ServiceBuilderPubSub = iox2lib
    .lookup<NativeFunction<_IoxServiceBuilderPubSubC>>(
        'iox2_service_builder_pub_sub')
    .asFunction<_IoxServiceBuilderPubSub>();

final iox2ServiceBuilderPubSubSetPayloadTypeDetails = iox2lib
    .lookup<NativeFunction<_IoxServiceBuilderPubSubSetPayloadTypeDetailsC>>(
        'iox2_service_builder_pub_sub_set_payload_type_details')
    .asFunction<_IoxServiceBuilderPubSubSetPayloadTypeDetails>();

final iox2ServiceBuilderPubSubOpenOrCreate = iox2lib
    .lookup<NativeFunction<_IoxServiceBuilderPubSubOpenOrCreateC>>(
        'iox2_service_builder_pub_sub_open_or_create')
    .asFunction<_IoxServiceBuilderPubSubOpenOrCreate>();

// Port factory functions
final iox2PortFactoryPubSubPublisherBuilder = iox2lib
    .lookup<NativeFunction<_IoxPortFactoryPubSubPublisherBuilderC>>(
        'iox2_port_factory_pub_sub_publisher_builder')
    .asFunction<_IoxPortFactoryPubSubPublisherBuilder>();

final iox2PortFactoryPublisherBuilderCreate = iox2lib
    .lookup<NativeFunction<_IoxPortFactoryPublisherBuilderCreateC>>(
        'iox2_port_factory_publisher_builder_create')
    .asFunction<_IoxPortFactoryPublisherBuilderCreate>();

final iox2PortFactoryPubSubSubscriberBuilder = iox2lib
    .lookup<NativeFunction<_IoxPortFactoryPubSubSubscriberBuilderC>>(
        'iox2_port_factory_pub_sub_subscriber_builder')
    .asFunction<_IoxPortFactoryPubSubSubscriberBuilder>();

final iox2PortFactorySubscriberBuilderCreate = iox2lib
    .lookup<NativeFunction<_IoxPortFactorySubscriberBuilderCreateC>>(
        'iox2_port_factory_subscriber_builder_create')
    .asFunction<_IoxPortFactorySubscriberBuilderCreate>();

// Publisher functions
final iox2PublisherLoanSliceUninit = iox2lib
    .lookup<NativeFunction<_IoxPublisherLoanSliceUninitC>>(
        'iox2_publisher_loan_slice_uninit')
    .asFunction<_IoxPublisherLoanSliceUninit>();

final iox2PublisherDrop = iox2lib
    .lookup<NativeFunction<_IoxPublisherDropC>>('iox2_publisher_drop')
    .asFunction<_IoxPublisherDrop>();

// Subscriber functions
final iox2SubscriberReceive = iox2lib
    .lookup<NativeFunction<_IoxSubscriberReceiveC>>('iox2_subscriber_receive')
    .asFunction<_IoxSubscriberReceive>();

final iox2SubscriberDrop = iox2lib
    .lookup<NativeFunction<_IoxSubscriberDropC>>('iox2_subscriber_drop')
    .asFunction<_IoxSubscriberDrop>();

// Sample functions
final iox2SampleMutPayloadMut = iox2lib
    .lookup<NativeFunction<_IoxSampleMutPayloadMutC>>(
        'iox2_sample_mut_payload_mut')
    .asFunction<_IoxSampleMutPayloadMut>();

final iox2SampleMutSend = iox2lib
    .lookup<NativeFunction<_IoxSampleMutSendC>>('iox2_sample_mut_send')
    .asFunction<_IoxSampleMutSend>();

final iox2SamplePayload = iox2lib
    .lookup<NativeFunction<_IoxSamplePayloadC>>('iox2_sample_payload')
    .asFunction<_IoxSamplePayload>();

final iox2SampleDrop = iox2lib
    .lookup<NativeFunction<_IoxSampleDropC>>('iox2_sample_drop')
    .asFunction<_IoxSampleDrop>();
