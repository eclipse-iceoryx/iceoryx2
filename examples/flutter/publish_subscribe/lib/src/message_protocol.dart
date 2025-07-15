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

/// Message protocol for iceoryx2 communication
/// This file defines message communication protocols and serialization/deserialization logic.
library message_protocol;

import 'dart:ffi';
import 'dart:convert';
import 'dart:typed_data';

import 'ffi/iceoryx2_ffi.dart' as ffi;

/// Message data structure for communication
class Message {
  final String content;
  final DateTime timestamp;
  final String sender;

  const Message({
    required this.content,
    required this.timestamp,
    required this.sender,
  });

  /// Create a message with current timestamp
  factory Message.create(String content, {String sender = 'unknown'}) {
    return Message(
      content: content,
      timestamp: DateTime.now(),
      sender: sender,
    );
  }

  @override
  String toString() =>
      'Message(content: "$content", sender: "$sender", timestamp: $timestamp)';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Message &&
          runtimeType == other.runtimeType &&
          content == other.content &&
          timestamp == other.timestamp &&
          sender == other.sender;

  @override
  int get hashCode => content.hashCode ^ timestamp.hashCode ^ sender.hashCode;
}

/// Helper class for message serialization and deserialization
///
/// Message format:
/// - First 8 bytes: message length (uint64)
/// - Next N bytes: JSON-encoded message data
/// - Remaining bytes: zero-padding to MESSAGE_STRUCT_SIZE
class MessageProtocol {
  /// Serialize a message to the structured format expected by iceoryx2
  static void serialize(Message message, Pointer<Uint8> buffer) {
    try {
      // Create JSON representation
      final messageData = {
        'content': message.content,
        'timestamp': message.timestamp.toIso8601String(),
        'sender': message.sender,
      };

      final jsonString = jsonEncode(messageData);
      final jsonBytes = utf8.encode(jsonString);

      if (jsonBytes.length > ffi.MESSAGE_MAX_LENGTH) {
        throw ArgumentError(
            'Message too long: ${jsonBytes.length} > ${ffi.MESSAGE_MAX_LENGTH}');
      }

      // Clear the entire buffer first
      for (int i = 0; i < ffi.MESSAGE_STRUCT_SIZE; i++) {
        buffer[i] = 0;
      }

      // Write message length (8 bytes, little-endian)
      final lengthBytes = ByteData(8);
      lengthBytes.setUint64(0, jsonBytes.length, Endian.little);
      for (int i = 0; i < 8; i++) {
        buffer[i] = lengthBytes.getUint8(i);
      }

      // Write message content
      for (int i = 0; i < jsonBytes.length; i++) {
        buffer[8 + i] = jsonBytes[i];
      }

      // Optional debug output
      // print('[MessageProtocol] Serialized message: ${jsonBytes.length} bytes');
    } catch (e) {
      print('[MessageProtocol] Serialization error: $e');
      rethrow;
    }
  }

  /// Deserialize a message from the structured format
  static Message deserialize(Pointer<Uint8> buffer) {
    try {
      // Read message length (8 bytes, little-endian)
      final lengthBytes = ByteData(8);
      for (int i = 0; i < 8; i++) {
        lengthBytes.setUint8(i, buffer[i]);
      }
      final messageLength = lengthBytes.getUint64(0, Endian.little);

      if (messageLength == 0) {
        throw FormatException('Empty message');
      }

      if (messageLength > ffi.MESSAGE_MAX_LENGTH) {
        throw FormatException(
            'Message too long: $messageLength > ${ffi.MESSAGE_MAX_LENGTH}');
      }

      // Read message content
      final messageBytes = Uint8List(messageLength.toInt());
      for (int i = 0; i < messageLength; i++) {
        messageBytes[i] = buffer[8 + i];
      }

      final jsonString = utf8.decode(messageBytes);
      final messageData = jsonDecode(jsonString) as Map<String, dynamic>;

      // Optional debug output
      // print('[MessageProtocol] Deserialized message: ${messageLength} bytes');

      return Message(
        content: messageData['content'] as String,
        timestamp: DateTime.parse(messageData['timestamp'] as String),
        sender: messageData['sender'] as String,
      );
    } catch (e) {
      print('[MessageProtocol] Deserialization error: $e');
      rethrow;
    }
  }
}
