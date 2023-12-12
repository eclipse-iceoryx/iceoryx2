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

//! Contains simplistic math functions.

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
    align(value, std::mem::align_of::<T>())
}

/// Calculates log2 of a number which is a power of 2
pub fn log2_of_power_of_2(value: u64) -> u8 {
    let mut bits = value;

    for i in 0..64 {
        if bits == 1 {
            return i;
        }

        bits >>= 1;
    }

    0
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
                result.push_str(&unsafe { core::mem::transmute::<[u8; 16], u128>(data) }.to_b64());
                i += 16;
            } else if 8 < N - i {
                let mut data = [0u8; 8];
                data.copy_from_slice(&self[i..i + 8]);
                result.push_str(&unsafe { core::mem::transmute::<[u8; 8], u64>(data) }.to_b64());
                i += 8;
            } else if 4 < N - i {
                let mut data = [0u8; 4];
                data.copy_from_slice(&self[i..i + 4]);
                result.push_str(&unsafe { core::mem::transmute::<[u8; 4], u32>(data) }.to_b64());
                i += 4;
            } else if 2 < N - i {
                let mut data = [0u8; 2];
                data.copy_from_slice(&self[i..i + 2]);
                result.push_str(&unsafe { core::mem::transmute::<[u8; 2], u16>(data) }.to_b64());
                i += 2;
            } else {
                result.push_str(&self[i].to_b64());
                i += 1;
            }
        }
        result
    }
}

pub fn round_to_pow2(mut value: u64) -> u64 {
    value -= 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value |= value >> 32;
    value += 1;

    value
}
