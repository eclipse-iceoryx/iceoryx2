# Using iceoryx2 with Zig

## How to use C FFI bindings from Zig

**Disclaimer:** Zig is pre-1.0 and has breaking changes every day.

The following applies to Zig 0.14.0.
This example is for cross-compiling to aarch64,
but you can compile for your host arch just by simply editing `target`.

### `build.zig`

```c
const std = @import("std");
// replace with your own absolute path
const iox2_root = "/full/path/to/.../iceoryx2/target/ff/cc/install";

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
        b.path("../path/to/iceoryx2/target/ff/cc/install/include/iceoryx2/v0.7.0/iox2/") // replace with your own relative path, this is where iceoryx2.h lives
    );
    // This line exe.linkSystemLibrary("gcc_s"); is needed for Rust unwind
    exe.linkSystemLibrary("gcc_s"); // Link to libgcc_s runtime resident on target
    exe.addObjectFile(b.path("../path/to/iceoryx2/target/ff/cc/install/lib/libiceoryx2_ffi.a")); // replace with your own relative path to the static ffi lib
    exe.linkLibC(); // For HTTP client
    
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
const iox2_root = "/full/path/to/.../iceoryx2/target/ff/cc/install";
const zig_proj_root = "/full/path/to/.../src";

const std = @import("std");

const iox2 = @cImport({
    @cInclude(iox2_root ++ "/include/iceoryx2/v0.7.0/iox2/iceoryx2.h");
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
