# Flutter Examples for iceoryx2

This directory contains Flutter examples demonstrating how to use iceoryx2 with Dart FFI.

## Prerequisites

1. **Flutter SDK**: Install Flutter SDK (https://flutter.dev/docs/get-started/install)
2. **iceoryx2 C FFI Library**: Build the iceoryx2 C FFI library first:
   ```bash
   cd ../../
   cargo build --release -p iceoryx2-ffi
   ```
3. **Generated Headers**: Ensure the C headers are generated:
   ```bash
   ls target/release/iceoryx2-ffi-cbindgen/include/iox2/iceoryx2.h
   ```

## Examples

### publish_subscribe
A simple publisher and subscriber example showing basic IPC communication.

**Usage:**
```bash
cd publish_subscribe

# Terminal 1 - Run subscriber
flutter run -d linux lib/subscriber.dart

# Terminal 2 - Run publisher  
flutter run -d linux lib/publisher.dart
```

## Notes

- These examples currently work on Linux desktop only
- Mobile platform support requires additional platform-specific configurations
- The examples use `dart:ffi` to call the iceoryx2 C API directly
- Error handling is simplified for demonstration purposes

## Building for Production

For production use, consider:
1. Adding proper error handling and logging
2. Implementing async/await patterns for non-blocking operations
3. Adding platform-specific build configurations
4. Creating higher-level Dart wrapper APIs
