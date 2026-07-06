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

/// Writes `profile` onto an rcl-provided default QoS. `None` durations keep
/// the rcl defaults.
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
        Reliability::BestAvailable => {
            rmw_qos_reliability_policy_e::RMW_QOS_POLICY_RELIABILITY_BEST_AVAILABLE
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
        Durability::BestAvailable => {
            rmw_qos_durability_policy_e::RMW_QOS_POLICY_DURABILITY_BEST_AVAILABLE
        }
    };
    qos.liveliness = match profile.liveliness {
        Liveliness::SystemDefault => {
            rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_SYSTEM_DEFAULT
        }
        Liveliness::Automatic => rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_AUTOMATIC,
        Liveliness::ManualByTopic => {
            rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_MANUAL_BY_TOPIC
        }
        Liveliness::BestAvailable => {
            rmw_qos_liveliness_policy_e::RMW_QOS_POLICY_LIVELINESS_BEST_AVAILABLE
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

fn time(duration: Duration) -> rmw_time_s {
    rmw_time_s {
        sec: duration.as_secs(),
        nsec: duration.subsec_nanos() as u64,
    }
}
