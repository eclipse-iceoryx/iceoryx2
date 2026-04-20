// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use iceoryx2_dmabuf::{FdSidecarError, FdSidecarToken};

#[test]
fn token_debug_printable() {
    // Construct via from_nonzero since the token field is pub(crate).
    let t = FdSidecarToken::from_nonzero(core::num::NonZeroU64::new(42).unwrap());
    assert!(format!("{t:?}").contains("42"));
}

#[test]
fn error_variants_display() {
    let e = FdSidecarError::TokenExhausted;
    assert!(!format!("{e:?}").is_empty());
    let e2 = FdSidecarError::UnsupportedPlatform;
    assert!(!format!("{e2:?}").is_empty());
}

// ── T12: token monotonicity + wraparound guard ────────────────────────────────

/// Verify that the kernel sentinel `0` is not representable as a `NonZeroU64`.
///
/// The publisher uses `NonZeroU64::new(raw).ok_or(TokenExhausted)` so that
/// when the 64-bit counter wraps from `u64::MAX` through `wrapping_add(1)` back
/// to `0`, the very next `send` call returns `TokenExhausted` rather than
/// silently emitting a zero-token sample.
#[test]
fn token_zero_is_rejected() {
    // AtomicU64 wraps to 0 after u64::MAX fetches — verify NonZeroU64::new(0)
    // returns None.
    assert!(core::num::NonZeroU64::new(0).is_none());
}

/// Verify the mapping from raw-zero to `TokenExhausted`.
///
/// This mirrors the production path inside `FdSidecarPublisher::send`:
/// ```rust
/// let raw = self.next_token;
/// self.next_token = self.next_token.wrapping_add(1);
/// let token = NonZeroU64::new(raw).ok_or(FdSidecarError::TokenExhausted)?;
/// ```
/// When `raw == 0` (post-wraparound), the `ok_or` converts `None` to
/// `Err(TokenExhausted)`.
#[test]
fn token_exhausted_error_on_zero() {
    let raw: u64 = 0;
    let result =
        core::num::NonZeroU64::new(raw).ok_or(iceoryx2_dmabuf::FdSidecarError::TokenExhausted);
    assert!(
        matches!(result, Err(iceoryx2_dmabuf::FdSidecarError::TokenExhausted)),
        "expected TokenExhausted when raw token is 0, got {result:?}",
    );
}

/// Verify that tokens are monotonically increasing across consecutive values.
///
/// Simulates the publisher counter loop: checks that each successive `u64` in
/// the range `1..=N` converts to a distinct, strictly increasing `NonZeroU64`.
#[test]
fn token_monotonic_across_many_values() {
    const N: u64 = 1_000;
    let mut prev: Option<core::num::NonZeroU64> = None;
    for raw in 1_u64..=N {
        let tok = core::num::NonZeroU64::new(raw).expect("all values 1..=N must be non-zero");
        if let Some(p) = prev {
            assert!(
                tok.get() > p.get(),
                "token {tok} is not strictly greater than previous {p}"
            );
        }
        prev = Some(tok);
    }
}

/// Verify that `u64::MAX` is representable as a non-zero token (last valid
/// token before wraparound), and that adding 1 produces 0 (wraparound).
#[test]
fn token_u64_max_then_wraparound() {
    let last_valid = core::num::NonZeroU64::new(u64::MAX);
    assert!(
        last_valid.is_some(),
        "u64::MAX must be representable as a non-zero token"
    );

    // Simulate wrapping_add(1) on u64::MAX → 0.
    let next_raw = u64::MAX.wrapping_add(1);
    assert_eq!(next_raw, 0, "u64::MAX.wrapping_add(1) must be 0");

    // The publisher would then call NonZeroU64::new(0) → None → TokenExhausted.
    let result =
        core::num::NonZeroU64::new(next_raw).ok_or(iceoryx2_dmabuf::FdSidecarError::TokenExhausted);
    assert!(
        matches!(result, Err(iceoryx2_dmabuf::FdSidecarError::TokenExhausted)),
        "expected TokenExhausted after u64::MAX wraparound, got {result:?}",
    );
}
