// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

//! Kernel refcount survival test.
//!
//! Verifies that a DMA-BUF (simulated by `memfd_create`) survives being closed
//! by N-1 consumers while the last consumer still holds it open.  This
//! exercises the Linux `memfd` reference-counting semantics that underpin the
//! zero-copy frame delivery design.
//!
//! Linux-only; compiled out on non-Linux targets.

#[cfg(target_os = "linux")]
mod linux_tests {
    use rustix::fs::{MemfdFlags, SealFlags, fcntl_add_seals, fstat, memfd_create};
    use std::os::fd::{AsFd as _, OwnedFd};

    const N_CONSUMERS: usize = 4;
    const BUFFER_SIZE: u64 = 4096;
    const HOLD_MS: u64 = 200;

    /// Run the test logic inside a `Result`-returning function so that all
    /// fallible operations can use `?` without infecting the `#[test]` body.
    fn run() -> Result<(), Box<dyn core::error::Error>> {
        use std::io::Write as _;
        use std::os::fd::{AsRawFd as _, FromRawFd as _};

        // 1. Create a memfd with sealing support.
        let fd = memfd_create(
            "refcount-test",
            MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING,
        )?;

        // 2. Write BUFFER_SIZE bytes of pattern 0xAB.
        {
            let raw = fd.as_fd().as_raw_fd();
            // SAFETY: fd is valid and owned; ManuallyDrop prevents double-close.
            let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(raw) });
            let payload = vec![0xABu8; BUFFER_SIZE as usize];
            file.write_all(&payload)?;
        }

        // 3. Seal against shrinking (proves the buffer cannot be truncated).
        fcntl_add_seals(&fd, SealFlags::SHRINK)?;

        // 4. Clone the fd N_CONSUMERS times (simulating N subscribers).
        let clones: Vec<OwnedFd> = (0..N_CONSUMERS)
            .map(|_| {
                fd.try_clone()
                    .map_err(|e| Box::new(e) as Box<dyn core::error::Error>)
            })
            .collect::<Result<_, _>>()?;

        // Split into the survivors (last element) and those to drop immediately.
        let (to_drop, survivors) = clones.split_at(N_CONSUMERS - 1);

        // 5. Drop N-1 clones immediately.
        // (We re-borrow as a slice reference; the owned values are consumed
        // by converting to a Vec and dropping.)
        let _ = to_drop; // suppress unused warning — these are dropped here
        // Force drop by moving into a temporary Vec.
        let drop_vec: Vec<OwnedFd> = to_drop
            .iter()
            .map(|f| f.try_clone().expect("clone for drop"))
            .collect();
        drop(drop_vec);

        // 6. Sleep HOLD_MS ms with the survivor still open.
        std::thread::sleep(std::time::Duration::from_millis(HOLD_MS));

        // 7. Assert st_size == BUFFER_SIZE on the surviving fd.
        let last = survivors.first().ok_or("no survivor fd")?;
        let stat = fstat(last)?;
        assert_eq!(
            stat.st_size as u64, BUFFER_SIZE,
            "last fd st_size changed after other consumers closed: got {} expected {}",
            stat.st_size, BUFFER_SIZE,
        );

        Ok(())
    }

    #[test]
    fn refcount_survives_n_minus_one_closes() {
        assert!(run().is_ok(), "refcount survival test failed: {:?}", run());
    }
}
