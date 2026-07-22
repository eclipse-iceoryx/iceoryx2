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

use core::time::Duration;

use r2r_rcl::{
    rmw_qos_durability_policy_e, rmw_qos_history_policy_e, rmw_qos_liveliness_policy_e,
    rmw_qos_profile_t, rmw_qos_reliability_policy_e, rmw_time_s,
};

use crate::qos::{Durability, History, Liveliness, QosProfile, Reliability};

/// Matches `RMW_DURATION_INFINITE`. Represents unbounded.
const INFINITE: rmw_time_s = rmw_time_s {
    sec: 9223372036,
    nsec: 854775807,
};

/// Applies a QosProfile onto the rcl representation.
pub(crate) fn apply(profile: &QosProfile, qos: &mut rmw_qos_profile_t) {
    match profile.history {
        History::SystemDefault => {
            qos.history = rmw_qos_history_policy_e::RMW_QOS_POLICY_HISTORY_SYSTEM_DEFAULT;
        }
        History::KeepLast(depth) => {
            qos.history = rmw_qos_history_policy_e::RMW_QOS_POLICY_HISTORY_KEEP_LAST;
            qos.depth = depth;
        }
        History::KeepAll => {
            qos.history = rmw_qos_history_policy_e::RMW_QOS_POLICY_HISTORY_KEEP_ALL;
        }
    }
    qos.reliability = match profile.reliability {
        Reliability::SystemDefault => {
            rmw_qos_reliability_policy_e::RMW_QOS_POLICY_RELIABILITY_SYSTEM_DEFAULT
        }
        Reliability::Reliable => rmw_qos_reliability_policy_e::RMW_QOS_POLICY_RELIABILITY_RELIABLE,
        Reliability::BestEffort => {
            rmw_qos_reliability_policy_e::RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT
        }
    };
    qos.durability = match profile.durability {
        Durability::SystemDefault => {
            rmw_qos_durability_policy_e::RMW_QOS_POLICY_DURABILITY_SYSTEM_DEFAULT
        }
        Durability::TransientLocal => {
            rmw_qos_durability_policy_e::RMW_QOS_POLICY_DURABILITY_TRANSIENT_LOCAL
        }
        Durability::Volatile => rmw_qos_durability_policy_e::RMW_QOS_POLICY_DURABILITY_VOLATILE,
    };
    qos.liveliness = match profile.liveliness {
        Liveliness::SystemDefault => {
            rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_SYSTEM_DEFAULT
        }
        Liveliness::Automatic => rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_AUTOMATIC,
        Liveliness::ManualByTopic => {
            rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_MANUAL_BY_TOPIC
        }
    };
    if let Some(deadline) = profile.deadline {
        qos.deadline = time(deadline);
    }
    if let Some(lifespan) = profile.lifespan {
        qos.lifespan = time(lifespan);
    }
    if let Some(lease) = profile.liveliness_lease_duration {
        qos.liveliness_lease_duration = time(lease);
    }
}

// Parses an rcl representation of QoS into a QosProfile.
pub(crate) fn parse(qos: &rmw_qos_profile_t) -> QosProfile {
    QosProfile {
        history: match qos.history {
            rmw_qos_history_policy_e::RMW_QOS_POLICY_HISTORY_KEEP_LAST => {
                History::KeepLast(qos.depth)
            }
            rmw_qos_history_policy_e::RMW_QOS_POLICY_HISTORY_KEEP_ALL => History::KeepAll,
            _ => History::SystemDefault,
        },
        reliability: match qos.reliability {
            rmw_qos_reliability_policy_e::RMW_QOS_POLICY_RELIABILITY_RELIABLE => {
                Reliability::Reliable
            }
            rmw_qos_reliability_policy_e::RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT => {
                Reliability::BestEffort
            }
            _ => Reliability::SystemDefault,
        },
        durability: match qos.durability {
            rmw_qos_durability_policy_e::RMW_QOS_POLICY_DURABILITY_TRANSIENT_LOCAL => {
                Durability::TransientLocal
            }
            rmw_qos_durability_policy_e::RMW_QOS_POLICY_DURABILITY_VOLATILE => Durability::Volatile,
            _ => Durability::SystemDefault,
        },
        deadline: duration(&qos.deadline),
        lifespan: duration(&qos.lifespan),
        liveliness: match qos.liveliness {
            rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_AUTOMATIC => {
                Liveliness::Automatic
            }
            rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_MANUAL_BY_TOPIC => {
                Liveliness::ManualByTopic
            }
            _ => Liveliness::SystemDefault,
        },
        liveliness_lease_duration: duration(&qos.liveliness_lease_duration),
    }
}

fn time(duration: Duration) -> rmw_time_s {
    rmw_time_s {
        sec: duration.as_secs(),
        nsec: duration.subsec_nanos() as u64,
    }
}

fn duration(time: &rmw_time_s) -> Option<Duration> {
    let unset = time.sec == 0 && time.nsec == 0;
    let infinite = (time.sec, time.nsec) >= (INFINITE.sec, INFINITE.nsec);
    (!unset && !infinite).then(|| Duration::from_secs(time.sec) + Duration::from_nanos(time.nsec))
}

#[cfg(test)]
mod tests {
    use super::*;

    use r2r_rcl::rcl_publisher_get_default_options;

    #[test]
    fn default_profile_matches_rmw_default() {
        let rmw_default = unsafe { rcl_publisher_get_default_options() }.qos;
        assert_eq!(parse(&rmw_default), QosProfile::default());
    }
}
