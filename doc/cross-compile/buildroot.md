# How to make cross-compilation work with buildroot

1. Install the build dependencies on your host PC, like: cmake, g++, clang...

2. Install the `rust` toolchain:

   ```console
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Add the arm64 target for Rust:

   ```console
   rustup target add aarch64-unknown-linux-gnu
   ```

4. Modify the arm64 target name to suit your cross-compilation tool, need to
   create this file `~/.cargo/config.toml`, add the below code,
   "aarch64-buildroot-linux-GNU-gcc" which is your real cross-compilation tool
   name:
   ```console
   [target.aarch64-unknown-linux-gnu]
   linker = "aarch64-buildroot-linux-gnu-gcc"
   ```
5. Source your cross-compilation buildroot environment:

   ```console
   source /to/your/environment-setup 
   ```

   The `environment-setup` file should be in your buildroot directory.

6. Add the buildroot sysroot on host PC environment:

   ```console
   export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/to/your/sysroot"
   ```

7. Change to the iceoryx2 directory

   ```console
   cd iceoryx2
   ```

8. Configure, build and install iceoryx2
   ```console
   cmake -S . -B build -DBUILD_EXAMPLES=ON -DCMAKE_INSTALL_PREFIX=../\_OUTPUT -DRUST_TARGET_TRIPLET='aarch64-unknown-linux-GNU'
   cmake --build build
   cmake --install build
   ```

Finally, you can get the arm64 libs, include files in the `_OUTPUT` directory.
