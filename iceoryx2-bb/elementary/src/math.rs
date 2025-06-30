// Copyright (c) 2023 - 2024 Contributors to the Eclipse Foundation
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

//! Contains simplistic math functions.

/// Returns the required memory size when alignment adjustments are taken into account
pub const fn unaligned_mem_size<T>(array_capacity: usize) -> usize {
    core::mem::size_of::<T>() * array_capacity + core::mem::align_of::<T>() - 1
}

/// Aligns value to alignment. It increments value to the next multiple of alignment.
pub const fn align(value: usize, alignment: usize) -> usize {
    if value % alignment == 0 {
        value
    } else {
        value + alignment - value % alignment
    }
}

/// Aligns value to the alignment of T.
pub const fn align_to<T>(value: usize) -> usize {
    align(value, core::mem::align_of::<T>())
}

pub trait ToB64 {
    fn to_b64(&self) -> String;
}

impl ToB64 for u128 {
    fn to_b64(&self) -> String {
        let mut quotient = *self;
        let mut remainder;
        let mut ret_val = String::new();

        let remainder_to_char = |value| -> char {
            if value < 10 {
                ((48 + value) as u8) as char
            } else if value < 26 + 10 {
                ((65 + value - 10) as u8) as char
            } else if value < 62 {
                ((97 + value - 10 - 26) as u8) as char
            } else if value == 62 {
                45 as char
            } else {
                95 as char
            }
        };

        loop {
            remainder = quotient % 64;
            quotient /= 64;

            ret_val.push(remainder_to_char(remainder));

            if quotient == 0 {
                break;
            }
        }

        ret_val
    }
}

impl ToB64 for u64 {
    fn to_b64(&self) -> String {
        (*self as u128).to_b64()
    }
}

impl ToB64 for u32 {
    fn to_b64(&self) -> String {
        (*self as u128).to_b64()
    }
}

impl ToB64 for u16 {
    fn to_b64(&self) -> String {
        (*self as u128).to_b64()
    }
}

impl ToB64 for u8 {
    fn to_b64(&self) -> String {
        (*self as u128).to_b64()
    }
}

impl<const N: usize> ToB64 for [u8; N] {
    fn to_b64(&self) -> String {
        let mut result = String::new();
        let mut i = 0;
        while N != i {
            if 16 < N - i {
                let mut data = [0u8; 16];
                data.copy_from_slice(&self[i..i + 16]);
                result.push_str(&u128::from_le_bytes(data).to_b64());
                i += 16;
            } else if 8 < N - i {
                let mut data = [0u8; 8];
                data.copy_from_slice(&self[i..i + 8]);
                result.push_str(&u64::from_le_bytes(data).to_b64());
                i += 8;
            } else if 4 < N - i {
                let mut data = [0u8; 4];
                data.copy_from_slice(&self[i..i + 4]);
                i += 8;
                result.push_str(&u32::from_le_bytes(data).to_b64());
                i += 4;
            } else if 2 < N - i {
                let mut data = [0u8; 2];
                data.copy_from_slice(&self[i..i + 2]);
                result.push_str(&u16::from_le_bytes(data).to_b64());
                i += 2;
            } else {
                result.push_str(&self[i].to_b64());
                i += 1;
            }
        }
        result
    }
}

/// Returns the larger of the two provided numbers
///
/// # Example
///
/// ```
/// use iceoryx2_bb_elementary::math::max;
///
/// const SIZE: usize = max(core::mem::size_of::<u32>(), core::mem::size_of::<u32>());
/// ```
///
/// # Note
///
/// Once const traits are stable, this should become a generic implementation for `T: PartialOrd`
pub const fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}
