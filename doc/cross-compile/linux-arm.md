# Cross-compiling iceoryx2

## Cross-compile project with `cross`

Setup `cross` as described here: [cross-rs GitHub](https://github.com/cross-rs/cross).

Once `cross` is setup, navigate to root of the iceoryx2 repo and run
the command below.

```bash
cross build --target aarch64-unknown-linux-gnu --release --package iceoryx2-ffi
```

## Build C bindings

Run the following at the repo root:

```bash
cmake -S . -B target/ffi/build -DBUILD_EXAMPLES=OFF -DCMAKE_INSTALL_PREFIX=target/ffi/install -DBUILD_CXX_BINDING=OFF -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/aarch64-unknown-linux-gnu/release"
```

## Build C examples

Download the correct version of the ARM toolchain here:
[ARM toolchain downloads](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads).
Make sure to read the release notes and ensure it matches with the
glibc version of your target.

Extract the archive, make note of where it is.

Make a cmake file. You can put it anywhere. We'll just call it `cross-example.cmake`

### `cross-example.cmake`

```cmake
set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

# absolute path to extracted ARM toolchain archive, it should look like this
set(TOOLCHAIN_ROOT "/full/path/to/.../arm-gnu-toolchain-12.2.rel1-x86_64-aarch64-none-linux-gnu")

set(CMAKE_C_COMPILER "${TOOLCHAIN_ROOT}/bin/aarch64-none-linux-gnu-gcc")
set(CMAKE_CXX_COMPILER "${TOOLCHAIN_ROOT}/bin/aarch64-none-linux-gnu-g++")

# libc and crt1.o live here
set(CMAKE_SYSROOT "${TOOLCHAIN_ROOT}/aarch64-none-linux-gnu/libc")

set(CMAKE_FIND_ROOT_PATH "${CMAKE_SYSROOT}")
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
```

Build using configuration in `cross-example.cmake`:

The following directories vary depending on your system:

`CMAKE_TOOLCHAIN_FILE` - Point to where `cross-example.cmake` is
located in your system

`CMAKE_PREFIX_PATH` - Point to where the `target/ffi/install` is
located within the cloned repo, just use an absolute path for your system

`iceoryx2-c_DIR` - Forces cmake to look for the iceoryx2-C headers

```bash
cmake -S examples/c/publish_subscribe \
  -B target/out-of-tree/examples/c/publish_subscribe \
  -DCMAKE_TOOLCHAIN_FILE="/full/path/to/.../cross-example.cmake" \
  -DCMAKE_PREFIX_PATH="/full/path/to/.../iceoryx2/target/ffi/install" \
  -Diceoryx2-c_DIR="/full/path/to/.../iceoryx2/target/ffi/install/lib/cmake/iceoryx2-c" \
  -DCMAKE_FIND_DEBUG_MODE=ON
```

```bash
cmake --build target/out-of-tree/examples/c/publish_subscribe
```

Your example binaries should be in `...target/out-of-tree/examples/c/publish_subscribe`

## Bonus: Build with Zig

Zig is pre-1.0 and has breaking changes every day.
The following applies to Zig 0.14.0.

### `build.zig`

```c
const std = @import("std");
// replace with your own absolute path
const iox2_root = "/full/path/to/.../iceoryx2/target/ffi/install";

pub fn build(b: *std.Build) void {

    const target = b.resolveTargetQuery(.{
            .cpu_arch = .aarch64,
            .os_tag = .linux,
            .abi = .gnu,
    });

    const optimize = b.standardOptimizeOption(.{});

    // Create your executable
    const exe = b.addExecutable(.{
        .name = "my_app",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    exe.addIncludePath(
        b.path("../path/to/iceoryx2/target/ffi/install/include/iceoryx2/v0.6.1/iox2/") // replace with your own relative path, this is where iceoryx2.h lives
    );
    exe.linkSystemLibrary("gcc_s"); // For Rust C unwinder. This is needed by iceoryx2.
    exe.addObjectFile(b.path("../path/to/iceoryx2/target/ffi/install/lib/libiceoryx2_ffi.a")); // replace with your own relative path to the static ffi lib
    exe.linkLibC();
    
    // Run step
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());
    
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run bot");
    run_step.dependOn(&run_cmd.step);
    b.installArtifact(exe); // you need this, or else you won't get a binary
}
```

### `main.zig`

This is a port of `subscriber.c`. Place `transmission_data.h` in the same
directory as `main.zig`.

```c
const iox2_root = "/full/path/to/.../iceoryx2/target/ffi/install";
const zig_proj_root = "/full/path/to/.../src";

const std = @import("std");

const iox2 = @cImport({
    @cInclude(iox2_root ++ "/include/iceoryx2/v0.6.1/iox2/iceoryx2.h");
    @cInclude(zig_proj_root ++ "/transmission_data.h");
});

pub fn main() !void {
    // create new node
    const node_builder_handle: iox2.iox2_node_builder_h = iox2.iox2_node_builder_new(null);
    var node_handle: iox2.iox2_node_h = null;
    if (iox2.iox2_node_builder_create(
            node_builder_handle,
            null,
            iox2.iox2_service_type_e_IPC,
            &node_handle
        ) != iox2.IOX2_OK)
    {
        std.log.err("Could not create node!\n", .{});
    }

    // create service name
    const service_name_value = "My/Funk/ServiceName";
    var service_name: iox2.iox2_service_name_h = null;
    if (iox2.iox2_service_name_new(
            null,
            service_name_value,
            service_name_value.len,
            &service_name
        ) != iox2.IOX2_OK)
    {
        std.log.err("Unable to create service name!\n", .{});
        iox2.iox2_node_drop(node_handle);
    }

    // create service builder
    const service_name_ptr: iox2.iox2_service_name_ptr = iox2.iox2_cast_service_name_ptr(service_name);
    const service_builder: iox2.iox2_service_builder_h = iox2.iox2_node_service_builder(&node_handle, null, service_name_ptr);
    const service_builder_pub_sub: iox2.iox2_service_builder_pub_sub_h = iox2.iox2_service_builder_pub_sub(service_builder);

    // set pub sub payload type
    const payload_type_name = "16TransmissionData";
    if (iox2.iox2_service_builder_pub_sub_set_payload_type_details(
            &service_builder_pub_sub,
            iox2.iox2_type_variant_e_FIXED_SIZE,
            payload_type_name,
            payload_type_name.len,
            @sizeOf(iox2.TransmissionData),
            @alignOf(iox2.TransmissionData)
        ) != iox2.IOX2_OK)
    {
        std.log.err("Unable to set type details\n", .{});
        iox2.iox2_service_name_drop(service_name);
    }

    // create service
    var service: iox2.iox2_port_factory_pub_sub_h = null;
    if (iox2.iox2_service_builder_pub_sub_open_or_create(
            service_builder_pub_sub,
            null, 
            &service) != iox2.IOX2_OK)
    {
        std.log.err("Unable to create service!\n", .{});
        iox2.iox2_service_name_drop(service_name);
    }

    // create subscriber
    const subscriber_builder: iox2.iox2_port_factory_subscriber_builder_h =
        iox2.iox2_port_factory_pub_sub_subscriber_builder(&service, null);
    var subscriber: iox2.iox2_subscriber_h = null;
    if (iox2.iox2_port_factory_subscriber_builder_create(subscriber_builder, null, &subscriber) != iox2.IOX2_OK) {
        std.log.err("Unable to create subscriber!\n", .{});
        iox2.iox2_service_name_drop(service_name);
    }

    while (iox2.iox2_node_wait(&node_handle, 1, 0) == iox2.IOX2_OK) {
        // receive sample
        var sample: iox2.iox2_sample_h = null;
        if (iox2.iox2_subscriber_receive(&subscriber, null, &sample) != iox2.IOX2_OK) {
            std.log.err("Failed to receive sample\n", .{});
            iox2.iox2_service_name_drop(service_name);
        }

        if (sample != null) {
            var payload: ?*iox2.TransmissionData = null;
            const casted_payload: *?*const anyopaque = @ptrCast(&payload);
            iox2.iox2_sample_payload(&sample, casted_payload, null);

            if (payload) |msg| {
                std.log.info("received: TransmissionData: .x: {}, .y: {}, .funky: {} \n",
                .{
                    msg.x,
                    msg.y,
                    msg.funky,
                });
            }
            iox2.iox2_sample_drop(sample);
        }
    }
}
```
