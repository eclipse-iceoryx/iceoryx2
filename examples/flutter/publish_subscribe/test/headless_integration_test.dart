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

/// Comprehensive headless test for iceoryx2 Flutter
/// Tests both publisher and subscriber functionality
void main() async {
  print('=== iceoryx2 Flutter Headless Integration Test ===');

  // Test 1: Segfault safety test
  await testSegfaultSafety();

  // Test 2: Publisher-Subscriber integration
  await testPublisherSubscriber();

  print('
PASSED All headless tests completed successfully!');
}

/// Test subscriber without publisher to ensure no segfaults
Future<void> testSegfaultSafety() async {
  print('
 Test 1: Segfault Safety (Subscriber without Publisher)');

  final subscriberProcess = await Process.start(
    'dart',
    ['lib/headless/headless_subscriber.dart'],
    workingDirectory: Directory.current.path,
  );

  // Monitor for crashes
  bool crashed = false;
  final completer = Completer<void>();

  subscriberProcess.exitCode.then((code) {
    if (code != 0 && code != 143 && code != 15) {
      // Exclude normal termination
      crashed = true;
      print('FAILED Subscriber crashed with exit code: $code');
    }
    if (!completer.isCompleted) completer.complete();
  });

  // Capture output
  subscriberProcess.stdout
      .transform(const SystemEncoding().decoder)
      .listen((data) {
    // Silently capture output - only show if there's an error
  });

  subscriberProcess.stderr
      .transform(const SystemEncoding().decoder)
      .listen((data) {
    if (data.toLowerCase().contains('segmentation') ||
        data.toLowerCase().contains('segfault')) {
      crashed = true;
      print(' Segmentation fault detected: $data');
    }
  });

  // Wait 5 seconds
  await Future.any([
    Future.delayed(const Duration(seconds: 5)),
    completer.future,
  ]);

  // Terminate gracefully
  subscriberProcess.kill(ProcessSignal.sigterm);

  if (!completer.isCompleted) {
    await subscriberProcess.exitCode.timeout(
      const Duration(seconds: 2),
      onTimeout: () {
        subscriberProcess.kill(ProcessSignal.sigkill);
        return -1;
      },
    );
  }

  if (!crashed) {
    print('PASSED Test 1 PASSED: No segmentation fault detected');
  } else {
    print('FAILED Test 1 FAILED: Subscriber crashed or segfaulted');
    exit(1);
  }
}

/// Test publisher and subscriber working together
Future<void> testPublisherSubscriber() async {
  print('
 Test 2: Publisher-Subscriber Integration');

  // Start publisher
  final publisherProcess = await Process.start(
    'dart',
    ['lib/headless/headless_publisher.dart'],
    workingDirectory: Directory.current.path,
  );

  // Wait for publisher to initialize
  await Future.delayed(const Duration(seconds: 2));

  // Start subscriber
  final subscriberProcess = await Process.start(
    'dart',
    ['lib/headless/headless_subscriber.dart'],
    workingDirectory: Directory.current.path,
  );

  int pubMessages = 0;
  int subMessages = 0;

  // Monitor publisher
  publisherProcess.stdout
      .transform(const SystemEncoding().decoder)
      .listen((data) {
    if (data.contains('OK Sent message #')) {
      pubMessages++;
    }
  });

  // Monitor subscriber
  subscriberProcess.stdout
      .transform(const SystemEncoding().decoder)
      .listen((data) {
    if (data.contains('OK') &&
        (data.contains('Manual poll received:') || data.contains('#'))) {
      subMessages++;
    }
  });

  // Run for 10 seconds
  await Future.delayed(const Duration(seconds: 10));

  // Cleanup
  publisherProcess.kill();
  subscriberProcess.kill();

  await publisherProcess.exitCode;
  await subscriberProcess.exitCode;

  print(
      ' Results: Publisher sent $pubMessages, Subscriber received $subMessages');

  if (subMessages > 0 && pubMessages > 0) {
    print('PASSED Test 2 PASSED: Message exchange successful');
  } else {
    print('FAILED Test 2 FAILED: No message exchange detected');
    exit(1);
  }
}
