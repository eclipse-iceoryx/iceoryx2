// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_camel_case_types)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

extern crate alloc;

mod api;
pub use api::*;

#[cfg(test)]
#[cfg(feature = "std")]
mod tests;

#[cfg(not(feature = "std"))]
mod no_std {
    use alloc::alloc::{GlobalAlloc, Layout};
    use iceoryx2_pal_posix::posix::{free, malloc};

    struct LibcAllocator;

    unsafe impl GlobalAlloc for LibcAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            malloc(layout.size()) as *mut u8
        }
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            free(ptr as *mut core::ffi::c_void)
        }
    }

    #[global_allocator]
    static GLOBAL: LibcAllocator = LibcAllocator;

    use core::panic::PanicInfo;
    use iceoryx2_bb_posix::signal::SignalHandler;
    use iceoryx2_bb_print::coutln;

    #[panic_handler]
    pub fn panic(info: &PanicInfo) -> ! {
        coutln!("");
        coutln!("╔═══════════════════════════════════════╗");
        coutln!("║           PANIC OCCURRED!             ║");
        coutln!("╚═══════════════════════════════════════╝");

        if let Some(location) = info.location() {
            coutln!("Location: {}:{}\n", location.file(), location.line());
        }

        coutln!("Message: {}\n", info);

        SignalHandler::abort();

        loop {
            core::hint::spin_loop();
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_eh_personality() {}
}
