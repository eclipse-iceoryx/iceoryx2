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

use core::fmt::Display;
use core::str::FromStr;

const FIELD_SEPARATOR: &str = ":";
const COMPONENT_SEPARATOR: &str = ",";
const DEFAULT_DEPTH: usize = 42;
const INFINITE_SEC: u64 = 9_223_372_036;
const INFINITE_NSEC: u64 = 854_775_807;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QosError {
    InvalidFieldCount,
    InvalidComponentCount,
    InvalidNumber,
    UnknownReliability,
    UnknownDurability,
    UnknownHistory,
    UnknownLiveliness,
}

impl core::fmt::Display for QosError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "QosError::{self:?}")
    }
}

impl core::error::Error for QosError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Reliability {
    SystemDefault = 0,
    Reliable = 1,
    BestEffort = 2,
    Unknown = 3,
    BestAvailable = 4,
}

impl Reliability {
    const VARIANTS: [Self; 5] = [
        Self::SystemDefault,
        Self::Reliable,
        Self::BestEffort,
        Self::Unknown,
        Self::BestAvailable,
    ];
}

impl TryFrom<u64> for Reliability {
    type Error = QosError;

    fn try_from(value: u64) -> Result<Self, QosError> {
        Self::VARIANTS
            .into_iter()
            .find(|variant| *variant as u64 == value)
            .ok_or(QosError::UnknownReliability)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Durability {
    SystemDefault = 0,
    TransientLocal = 1,
    Volatile = 2,
    Unknown = 3,
    BestAvailable = 4,
}

impl Durability {
    const VARIANTS: [Self; 5] = [
        Self::SystemDefault,
        Self::TransientLocal,
        Self::Volatile,
        Self::Unknown,
        Self::BestAvailable,
    ];
}

impl TryFrom<u64> for Durability {
    type Error = QosError;

    fn try_from(value: u64) -> Result<Self, QosError> {
        Self::VARIANTS
            .into_iter()
            .find(|variant| *variant as u64 == value)
            .ok_or(QosError::UnknownDurability)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum History {
    SystemDefault = 0,
    KeepLast = 1,
    KeepAll = 2,
    Unknown = 3,
}

impl History {
    const VARIANTS: [Self; 4] = [
        Self::SystemDefault,
        Self::KeepLast,
        Self::KeepAll,
        Self::Unknown,
    ];
}

impl TryFrom<u64> for History {
    type Error = QosError;

    fn try_from(value: u64) -> Result<Self, QosError> {
        Self::VARIANTS
            .into_iter()
            .find(|variant| *variant as u64 == value)
            .ok_or(QosError::UnknownHistory)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Liveliness {
    SystemDefault = 0,
    Automatic = 1,
    ManualByNode = 2,
    ManualByTopic = 3,
    Unknown = 4,
    BestAvailable = 5,
}

impl Liveliness {
    const VARIANTS: [Self; 6] = [
        Self::SystemDefault,
        Self::Automatic,
        Self::ManualByNode,
        Self::ManualByTopic,
        Self::Unknown,
        Self::BestAvailable,
    ];
}

impl TryFrom<u64> for Liveliness {
    type Error = QosError;

    fn try_from(value: u64) -> Result<Self, QosError> {
        Self::VARIANTS
            .into_iter()
            .find(|variant| *variant as u64 == value)
            .ok_or(QosError::UnknownLiveliness)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RmwDuration {
    pub sec: u64,
    pub nsec: u64,
}

impl RmwDuration {
    pub const INFINITE: Self = Self {
        sec: INFINITE_SEC,
        nsec: INFINITE_NSEC,
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct QosProfile {
    pub reliability: Reliability,
    pub durability: Durability,
    pub history: History,
    pub depth: usize,
    pub deadline: RmwDuration,
    pub lifespan: RmwDuration,
    pub liveliness: Liveliness,
    pub liveliness_lease_duration: RmwDuration,
}

impl Default for QosProfile {
    /// The default profile of `rmw_zenoh_cpp`.
    fn default() -> Self {
        Self {
            reliability: Reliability::Reliable,
            durability: Durability::Volatile,
            history: History::KeepLast,
            depth: DEFAULT_DEPTH,
            deadline: RmwDuration::INFINITE,
            lifespan: RmwDuration::INFINITE,
            liveliness: Liveliness::Automatic,
            liveliness_lease_duration: RmwDuration::INFINITE,
        }
    }
}

fn format_value<T: PartialEq + Display>(value: T, default: T) -> String {
    if value == default {
        String::new()
    } else {
        value.to_string()
    }
}

fn format_duration(value: RmwDuration, default: RmwDuration) -> String {
    format!(
        "{}{COMPONENT_SEPARATOR}{}",
        format_value(value.sec, default.sec),
        format_value(value.nsec, default.nsec)
    )
}

fn parse_value<T: FromStr>(value: &str, default: T) -> Result<T, QosError> {
    if value.is_empty() {
        return Ok(default);
    }
    value.parse().map_err(|_| QosError::InvalidNumber)
}

fn parse_duration(sec: &str, nsec: &str, default: RmwDuration) -> Result<RmwDuration, QosError> {
    Ok(RmwDuration {
        sec: parse_value(sec, default.sec)?,
        nsec: parse_value(nsec, default.nsec)?,
    })
}

fn components<const N: usize>(field: &str) -> Result<[&str; N], QosError> {
    field
        .split(COMPONENT_SEPARATOR)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| QosError::InvalidComponentCount)
}

impl Display for QosProfile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let default = Self::default();
        write!(
            f,
            "{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{COMPONENT_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{COMPONENT_SEPARATOR}{}",
            format_value(self.reliability as u64, default.reliability as u64),
            format_value(self.durability as u64, default.durability as u64),
            format_value(self.history as u64, default.history as u64),
            format_value(self.depth, default.depth),
            format_duration(self.deadline, default.deadline),
            format_duration(self.lifespan, default.lifespan),
            format_value(self.liveliness as u64, default.liveliness as u64),
            format_duration(
                self.liveliness_lease_duration,
                default.liveliness_lease_duration
            ),
        )
    }
}

impl FromStr for QosProfile {
    type Err = QosError;

    fn from_str(qos: &str) -> Result<Self, QosError> {
        let default = Self::default();
        let [
            reliability,
            durability,
            history,
            deadline,
            lifespan,
            liveliness,
        ] = qos
            .split(FIELD_SEPARATOR)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| QosError::InvalidFieldCount)?;
        let [history, depth] = components(history)?;
        let [deadline_sec, deadline_nsec] = components(deadline)?;
        let [lifespan_sec, lifespan_nsec] = components(lifespan)?;
        let [liveliness, lease_sec, lease_nsec] = components(liveliness)?;

        Ok(Self {
            reliability: Reliability::try_from(parse_value(
                reliability,
                default.reliability as u64,
            )?)?,
            durability: Durability::try_from(parse_value(durability, default.durability as u64)?)?,
            history: History::try_from(parse_value(history, default.history as u64)?)?,
            depth: parse_value(depth, default.depth)?,
            deadline: parse_duration(deadline_sec, deadline_nsec, default.deadline)?,
            lifespan: parse_duration(lifespan_sec, lifespan_nsec, default.lifespan)?,
            liveliness: Liveliness::try_from(parse_value(liveliness, default.liveliness as u64)?)?,
            liveliness_lease_duration: parse_duration(
                lease_sec,
                lease_nsec,
                default.liveliness_lease_duration,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEPTH_ONE: usize = 1;
    const DEPTH_FIVE: usize = 5;
    const DEPTH_SEVEN: usize = 7;
    const DEPTH_TEN: usize = 10;
    const DEADLINE_SEC: u64 = 1;
    const LIFESPAN_NSEC: u64 = 500;
    const LEASE_SEC: u64 = 2;
    const LEASE_NSEC: u64 = 3;

    fn parse(qos: &str) -> Result<QosProfile, QosError> {
        qos.parse()
    }

    #[test]
    fn default_profile_is_all_empty() {
        assert_eq!(QosProfile::default().to_string(), "::,:,:,:,,");
        assert_eq!(parse("::,:,:,:,,"), Ok(QosProfile::default()));
    }

    #[test]
    fn captured_strings_round_trip() {
        assert_eq!(
            parse("::,7:,:,:,,").map(|q| q.to_string()),
            Ok("::,7:,:,:,,".to_string())
        );
        assert_eq!(
            parse("::,10:,:,:,,").map(|q| q.to_string()),
            Ok("::,10:,:,:,,".to_string())
        );
        assert_eq!(
            parse("2::,10:,:,:,,").map(|q| q.to_string()),
            Ok("2::,10:,:,:,,".to_string())
        );
        assert_eq!(
            parse("2::,5:,:,:,,").map(|q| q.to_string()),
            Ok("2::,5:,:,:,,".to_string())
        );
        assert_eq!(
            parse(":1:,1:,:,:,,").map(|q| q.to_string()),
            Ok(":1:,1:,:,:,,".to_string())
        );
        assert_eq!(
            parse("::2,10:,:,:,,").map(|q| q.to_string()),
            Ok("::2,10:,:,:,,".to_string())
        );
        assert_eq!(
            parse(":1:2,1:,:,:,,").map(|q| q.to_string()),
            Ok(":1:2,1:,:,:,,".to_string())
        );
        assert_eq!(
            parse("::,1000:,:,:,,").map(|q| q.to_string()),
            Ok("::,1000:,:,:,,".to_string())
        );
        assert_eq!(
            parse(":1:,1000:,:10,0:,,").map(|q| q.to_string()),
            Ok(":1:,1000:,:10,0:,,".to_string())
        );
    }

    #[test]
    fn parses_depth() {
        assert_eq!(
            parse("::,7:,:,:,,"),
            Ok(QosProfile {
                depth: DEPTH_SEVEN,
                ..QosProfile::default()
            })
        );
    }

    #[test]
    fn parses_sensor_data() {
        assert_eq!(
            parse("2::,5:,:,:,,"),
            Ok(QosProfile {
                reliability: Reliability::BestEffort,
                depth: DEPTH_FIVE,
                ..QosProfile::default()
            })
        );
    }

    #[test]
    fn parses_transient_local() {
        assert_eq!(
            parse(":1:,1:,:,:,,"),
            Ok(QosProfile {
                durability: Durability::TransientLocal,
                depth: DEPTH_ONE,
                ..QosProfile::default()
            })
        );
    }

    #[test]
    fn parses_keep_all() {
        assert_eq!(
            parse("::2,10:,:,:,,"),
            Ok(QosProfile {
                history: History::KeepAll,
                depth: DEPTH_TEN,
                ..QosProfile::default()
            })
        );
    }

    #[test]
    fn durations_and_liveliness_round_trip() {
        let qos = QosProfile {
            deadline: RmwDuration {
                sec: DEADLINE_SEC,
                ..RmwDuration::INFINITE
            },
            lifespan: RmwDuration {
                nsec: LIFESPAN_NSEC,
                ..RmwDuration::INFINITE
            },
            liveliness: Liveliness::ManualByTopic,
            liveliness_lease_duration: RmwDuration {
                sec: LEASE_SEC,
                nsec: LEASE_NSEC,
            },
            ..QosProfile::default()
        };

        assert_eq!(qos.to_string(), "::,:1,:,500:3,2,3");
        assert_eq!(parse("::,:1,:,500:3,2,3"), Ok(qos));
    }

    #[test]
    fn wrong_field_count_is_rejected() {
        for input in ["", "::,:,:,,", "::,:,:,:,,:", "::,:,:,:,,::"] {
            assert_eq!(parse(input), Err(QosError::InvalidFieldCount), "{input}");
        }
    }

    #[test]
    fn wrong_component_count_is_rejected() {
        for input in [
            "::7:,:,:,,",
            "::,7,:,:,:,,",
            ":::,:,:,,",
            "::,:,,:,:,,",
            "::,::,:,,",
            "::,:,:,,,:,,",
            "::,:,:,:,",
            "::,:,:,:,,,",
        ] {
            assert_eq!(
                parse(input),
                Err(QosError::InvalidComponentCount),
                "{input}"
            );
        }
    }

    #[test]
    fn non_numeric_values_are_rejected() {
        for input in [
            "x::,:,:,:,,",
            "::,-1:,:,:,,",
            "::,7.5:,:,:,,",
            "::,:-1,:,:,,",
            "::,:,:x,:,,",
            "::,:,:,:,18446744073709551616,",
        ] {
            assert_eq!(parse(input), Err(QosError::InvalidNumber), "{input}");
        }
    }

    #[test]
    fn unknown_reliability_is_rejected() {
        assert_eq!(parse("5::,:,:,:,,"), Err(QosError::UnknownReliability));
    }

    #[test]
    fn unknown_durability_is_rejected() {
        assert_eq!(parse(":5:,:,:,:,,"), Err(QosError::UnknownDurability));
    }

    #[test]
    fn unknown_history_is_rejected() {
        assert_eq!(parse("::4,:,:,:,,"), Err(QosError::UnknownHistory));
    }

    #[test]
    fn unknown_liveliness_is_rejected() {
        assert_eq!(parse("::,:,:,:6,,"), Err(QosError::UnknownLiveliness));
    }
}
