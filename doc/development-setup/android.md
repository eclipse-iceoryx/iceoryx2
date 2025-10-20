# Android Development Environment

> [!IMPORTANT]
> The MVP work only with the `local` and `local_threadsafe` service variant.
> The `ipc` and `ipc_threadsafe` service variants need more work to circumvent
> the Android sandbox limitations.

## Get and Setup Android

Install the Android Rust target:

```bash
rustup target add aarch64-linux-android x86_64-linux-android
```

This tool simplifies building Rust for Android but is not required:

```bash
cargo install cargo-ndk
```

The Android NDK is required in order to build Rust Android applications:

```bash
cd /opt
sudo mkdir android
sudo chown $USER:$USER android
cd android
wget https://dl.google.com/android/repository/android-ndk-r29-linux.zip
unizp android-ndk-r29-linux.zip     # unzips to 'android-ndk-r29'
```

In order for create binaries, the linker must be specified to cargo. Since the
path to the Android NDK might vary from user to user, we cannot set the path
in iceoryx2's `.cargo/config.toml`. Since this is depending on the users setup,
the linker should be added to `~/.cargo/config.toml`:
```toml
[target.x86_64-linux-android]
linker = "/opt/android/android-ndk-r29/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android35-clang"
[target.aarch64-linux-android]
linker = "/opt/android/android-ndk-r29/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android35-clang"
```

## ADB and Waydroid

`Waydroid` is used to run Android and `adb` to deploy an run the applications.

### Install ADB

On Arch Linux, `adb` can be installed with the following command:
```bash
sudo pacman -S android-tools
```

On Ubuntu Linux, `adb` can be installed with the following command:
```bash
sudo apt install android-tools-adb android-tools-fastboot
```

### Install and Setup Waydroid

`waydroid` is not available via the official Ubuntu repositories. This guide
assumes Arch Linux as OS. For detailed instructions please have a look at
https://wiki.archlinux.org/title/Waydroid.

`waydroid` is available via the official Arch repository. It can be installed
with `pacman` with the following command:
```bash
pacman -S waydroid
```

Once installed, `waydroid` needs to be initialized. This needs to be done only
once and it will download the Android image:
```bash
sudo waydroid init
```

In order to have some sane default for the Android window, it is recommended to
set the following properties:
```bash
waydroid prop set persist.waydroid.width 576
waydroid prop set persist.waydroid.height 1024
waydroid prop set persist.waydroid.suspend false
# potentially this needs to be done in order to activate the props
systemctl restart waydroid-container.service
```

The `suspend` property prevents the Android session to activate the lock-screen.
Without turning suspend off, the `adb` commands will fail a few minutes after
the last interaction with `waydroid`. Alternatively, the option could als be set
via `adb` with:
```bash
adb shell settings put global stay_on_while_plugged_in 3
```

It is recommended to use the `waydroid` property, though, and only use the `adb`
setting as fallback if the `waydroid` property does not work.

Now, `waydroid` can be used to start an Android session with:
```bash
waydroid session start
```

In a separate terminal, the available Android application can be listed and an
application can be run, e.g. the calculator:
```bash
waydroid app list
waydroid app launch com.android.calculator2
```

## Build and Run a Rust hello-world Application on Android

Create a hello world application:
```bash
cargo new --bin hello-world-android
cd hello-world-android
cargo build --target x86_64-linux-android --release
```

Copy the binary to `waydroid` (if `adb` hangs, deactivate the suspend property
in `waydroid`):
```bash
adb push target/x86_64-linux-android/release/hello-world-android /data/local/tmp
adb shell /data/local/tmp/hello-world-android
```

Alternatively, after `adb push ...` run `waydroid shell` in a terminal:
```bash
sudo waydroid shell
cd /data/local/tmp
./hello-world-android
```

## Build iceoryx2

Currently, only a subset of the iceoryx2 workspace can be build for Android:
```bash
cargo build --target x86_64-linux-android --package iceoryx2
```

Since only the `local` and `local_threadsafe` service variants are supported
for now, the `service_types_local_pubsub` example can be used to verify that
iceoryx2 indeed works on Android:
```bash
cargo build --example service_types_local_pubsub --target x86_64-linux-android
adb push target/x86_64-linux-android/debug/examples/service_types_local_pubsub /data/local/tmp
adb shell /data/local/tmp/service_types_local_pubsub
```

In order to build the `iceoryx2-zenoh-tunnel` the `CC` environment variable must
point to `clang` from the Android NDK:
```bash
export CC=/opt/android/android-ndk-r29/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android35-clang
cargo build --target x86_64-linux-android --package iceoryx2-tunnel-zenoh
```

In order to make the tunnel functional, more work is required.

The C and C++ bindings are not yet available for Android but once they work,
the procedure would be like this:

For the C and C++ bindings and examples, the `CC` and `CXX` environment variables
must point to `clang` and `clang++` from the Android NDK:
```bash
export CC=/opt/android/android-ndk-r29/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android35-clang
export CXX=/opt/android/android-ndk-r29/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android35-clang++
cargo build --target x86_64-linux-android --package iceoryx2-ffi-c --release
cmake -S . \
      -B target/ff/android/build \
      -DBUILD_EXAMPLES=ON \
      -DRUST_BUILD_ARTIFACT_PATH="$(pwd)/target/x86_64-linux-android/release"
```
