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

use core::convert::From;
use core::convert::TryFrom;
use core::{fmt::Debug, fmt::Display};

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;

#[derive(Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[repr(C)]
pub struct Ipv4Address(u32);

pub const LOCALHOST: Ipv4Address = Ipv4Address::new(127, 0, 0, 1);
pub const UNSPECIFIED: Ipv4Address = Ipv4Address::new(0, 0, 0, 0);
pub const BROADCAST: Ipv4Address = Ipv4Address::new(255, 255, 255, 255);

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error type returned when parsing an IPv4 address from a string.
pub enum Ipv4AddressParseError {
    // Returned when one of the octets is not a valid `u8` number (non-numeric or out-of-range).
    ParseIntError(core::num::ParseIntError),
    /// Returned when the input does not have exactly four parts separated by dots (e.g., too many or too few parts).
    WrongFormat,
}

impl From<core::num::ParseIntError> for Ipv4AddressParseError {
    fn from(value: core::num::ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl TryFrom<&str> for Ipv4Address {
    type Error = Ipv4AddressParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let msg = "Unable to construct IPv4 address from";
        let origin = "Ipv4Address::try_from()";
        let mut ipv4 = [0u8; 4];
        let mut counter = 0;
        for (n, value) in input.split(".").enumerate() {
            if n == 4 {
                fail!(from origin, with Ipv4AddressParseError::WrongFormat,
                    "{msg} \"{input}\" since the address contains too many parts.");
            }
            ipv4[n] = fail!(from origin, when value.parse::<u8>(),
                            "{msg} \"{input}\" since some entries are not an u8 number.");
            counter += 1;
        }

        if counter != 4 {
            fail!(from origin, with Ipv4AddressParseError::WrongFormat,
                "{msg} \"{input}\" since the address is incomplete.");
        }

        Ok(Self::new(ipv4[0], ipv4[1], ipv4[2], ipv4[3]))
    }
}

impl Ipv4Address {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | d as u32)
    }

    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    pub const fn octets(&self) -> [u8; 4] {
        [
            (self.0 >> 24) as u8,
            ((self.0 << 8) >> 24) as u8,
            ((self.0 << 16) >> 24) as u8,
            ((self.0 << 24) >> 24) as u8,
        ]
    }

    pub const fn is_unspecified(&self) -> bool {
        self.compare([0, 0, 0, 0], 32)
    }

    pub const fn is_loopback(&self) -> bool {
        self.compare([127, 0, 0, 0], 8)
    }

    pub const fn is_private(&self) -> bool {
        self.compare([10, 0, 0, 0], 8)
            || self.compare([192, 168, 0, 0], 16)
            || self.compare([172, 16, 0, 0], 12)
    }

    pub const fn is_link_local(&self) -> bool {
        self.compare([169, 254, 0, 0], 16)
    }

    pub const fn is_shared(&self) -> bool {
        self.compare([100, 64, 0, 0], 10)
    }

    pub const fn is_benchmarking(&self) -> bool {
        self.compare([198, 18, 0, 0], 15)
    }

    pub const fn is_reserved(&self) -> bool {
        self.compare([240, 0, 0, 0], 4) && !self.is_broadcast()
    }

    pub const fn is_multicast(&self) -> bool {
        self.compare([224, 0, 0, 0], 4)
    }

    pub const fn is_broadcast(&self) -> bool {
        self.compare(BROADCAST.octets(), 32)
    }

    pub const fn is_documentation(&self) -> bool {
        self.compare([192, 0, 2, 0], 24)
            || self.compare([198, 51, 100, 0], 24)
            || self.compare([203, 0, 113, 0], 24)
    }

    pub const fn is_global(&self) -> bool {
        !self.is_unspecified()
            && !self.is_private()
            && !self.is_shared()
            && !self.is_loopback()
            && !self.is_link_local()
            && !self.is_documentation()
            && !self.is_benchmarking()
            && !self.is_reserved()
            && !self.is_broadcast()
    }

    const fn compare(&self, value: [u8; 4], netmask: u8) -> bool {
        let rhs = Ipv4Address::new(value[0], value[1], value[2], value[3]);
        let shift = 32 - netmask;

        let rhs = rhs.0 >> shift;
        let lhs = self.0 >> shift;

        lhs == rhs
    }
}

impl Debug for Ipv4Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ipv4 = self.octets();
        write!(
            f,
            "Ipv4Address {{ {}.{}.{}.{} }}",
            ipv4[0], ipv4[1], ipv4[2], ipv4[3]
        )
    }
}

impl Display for Ipv4Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ipv4 = self.octets();
        write!(f, "{}.{}.{}.{}", ipv4[0], ipv4[1], ipv4[2], ipv4[3])
    }
}
