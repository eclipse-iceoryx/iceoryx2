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

//! The ROS 2 graph as the liveliness tokens of `rmw_zenoh_cpp` announce it.

use std::collections::BTreeMap;

use crate::codec::{EntityKind, Token, TokenError, TopicInfo};

#[derive(Debug, Default)]
pub(crate) struct Graph {
    entities: BTreeMap<String, Token>,
}

impl Graph {
    /// Adds the entity of a token, true when the graph changed.
    pub(crate) fn put(&mut self, key: &str) -> Result<bool, TokenError> {
        let token = Token::parse(key)?;
        Ok(self.entities.insert(key.to_string(), token).is_none())
    }

    /// Removes the entity of a token, true when the graph changed.
    pub(crate) fn delete(&mut self, key: &str) -> bool {
        self.entities.remove(key).is_some()
    }

    /// The topics of the publishers and subscriptions whose token passes `filter`.
    pub(crate) fn topics(
        &self,
        filter: impl Fn(&Token) -> bool,
    ) -> BTreeMap<&str, Vec<&TopicInfo>> {
        self.list(filter, [EntityKind::Publisher, EntityKind::Subscription])
    }

    /// The services of the servers and clients whose token passes `filter`.
    pub(crate) fn services(
        &self,
        filter: impl Fn(&Token) -> bool,
    ) -> BTreeMap<&str, Vec<&TopicInfo>> {
        self.list(
            filter,
            [EntityKind::ServiceServer, EntityKind::ServiceClient],
        )
    }

    fn list(
        &self,
        filter: impl Fn(&Token) -> bool,
        kinds: [EntityKind; 2],
    ) -> BTreeMap<&str, Vec<&TopicInfo>> {
        let mut listing: BTreeMap<&str, Vec<&TopicInfo>> = BTreeMap::new();
        for kind in kinds {
            for topic in self
                .entities
                .values()
                .filter(|token| token.kind == kind && filter(token))
                .filter_map(|token| token.topic.as_ref())
            {
                listing.entry(topic.name.as_str()).or_default().push(topic);
            }
        }
        listing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZID: &str = "6eeb7dddf6026d46786c8f11706568af";
    const OTHER_ZID: &str = "14cad03f5fb80bfe4d25373a53747a71";
    const PUBLISHER: &str = "MP";
    const SUBSCRIPTION: &str = "MS";
    const SERVICE_SERVER: &str = "SS";
    const SERVICE_CLIENT: &str = "SC";
    const CHATTER: &str = "/chatter";
    const OTHER_TOPIC: &str = "/status";
    const ADD_TWO_INTS: &str = "/add_two_ints";
    const NODE: &str = "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%/talker";
    const INVALID: &str = "@ros2_lv/x/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%/talker";
    const FIRST_DEPTH: usize = 7;
    const SECOND_DEPTH: usize = 8;
    const THIRD_DEPTH: usize = 9;
    const FOURTH_DEPTH: usize = 10;

    fn key(zid: &str, kind: &str, name: &str, depth: usize) -> String {
        format!(
            "@ros2_lv/0/{zid}/0/11/{kind}/%/%/talker/{}/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18/::,{depth}:,:,:,,",
            name.replace('/', "%")
        )
    }

    fn depths<'a>(listing: BTreeMap<&'a str, Vec<&TopicInfo>>) -> BTreeMap<&'a str, Vec<usize>> {
        listing
            .into_iter()
            .map(|(name, topics)| (name, topics.iter().map(|t| t.qos.depth).collect()))
            .collect()
    }

    fn listed<'a>(entries: &[(&'a str, &[usize])]) -> BTreeMap<&'a str, Vec<usize>> {
        entries
            .iter()
            .map(|(name, depths)| (*name, depths.to_vec()))
            .collect()
    }

    #[test]
    fn put_of_a_new_token_changes_the_graph() {
        let mut graph = Graph::default();

        assert_eq!(
            graph.put(&key(ZID, PUBLISHER, CHATTER, FIRST_DEPTH)),
            Ok(true)
        );
        assert_eq!(
            depths(graph.topics(|_| true)),
            listed(&[(CHATTER, &[FIRST_DEPTH])])
        );
    }

    #[test]
    fn repeated_put_does_not_change_the_graph() {
        let mut graph = Graph::default();
        let publisher = key(ZID, PUBLISHER, CHATTER, FIRST_DEPTH);

        assert_eq!(graph.put(&publisher), Ok(true));
        assert_eq!(graph.put(&publisher), Ok(false));
        assert_eq!(
            depths(graph.topics(|_| true)),
            listed(&[(CHATTER, &[FIRST_DEPTH])])
        );
    }

    #[test]
    fn invalid_key_is_an_error_and_changes_nothing() {
        let mut graph = Graph::default();

        assert_eq!(graph.put(INVALID), Err(TokenError::InvalidDomainId));
        assert!(!graph.delete(INVALID));
    }

    #[test]
    fn delete_of_a_known_token_changes_the_graph() {
        let mut graph = Graph::default();
        let publisher = key(ZID, PUBLISHER, CHATTER, FIRST_DEPTH);
        let other = key(ZID, PUBLISHER, OTHER_TOPIC, SECOND_DEPTH);

        assert_eq!(graph.put(&publisher), Ok(true));
        assert_eq!(graph.put(&other), Ok(true));
        assert!(graph.delete(&publisher));
        assert_eq!(
            depths(graph.topics(|_| true)),
            listed(&[(OTHER_TOPIC, &[SECOND_DEPTH])])
        );
    }

    #[test]
    fn delete_of_an_unknown_token_does_not_change_the_graph() {
        let mut graph = Graph::default();
        let publisher = key(ZID, PUBLISHER, CHATTER, FIRST_DEPTH);

        assert_eq!(graph.put(&publisher), Ok(true));
        assert!(graph.delete(&publisher));
        assert!(!graph.delete(&publisher));
    }

    #[test]
    fn node_token_is_in_neither_listing() {
        let mut graph = Graph::default();

        assert_eq!(graph.put(NODE), Ok(true));
        assert_eq!(depths(graph.topics(|_| true)), listed(&[]));
        assert_eq!(depths(graph.services(|_| true)), listed(&[]));
    }

    #[test]
    fn topics_and_services_are_split_by_kind() {
        let mut graph = Graph::default();

        for token in [
            key(ZID, PUBLISHER, CHATTER, FIRST_DEPTH),
            key(ZID, SUBSCRIPTION, CHATTER, SECOND_DEPTH),
            key(ZID, SERVICE_SERVER, ADD_TWO_INTS, THIRD_DEPTH),
            key(ZID, SERVICE_CLIENT, ADD_TWO_INTS, FOURTH_DEPTH),
        ] {
            assert_eq!(graph.put(&token), Ok(true));
        }

        assert_eq!(
            depths(graph.topics(|_| true)),
            listed(&[(CHATTER, &[FIRST_DEPTH, SECOND_DEPTH])])
        );
        assert_eq!(
            depths(graph.services(|_| true)),
            listed(&[(ADD_TWO_INTS, &[THIRD_DEPTH, FOURTH_DEPTH])])
        );
    }

    #[test]
    fn filter_hides_only_the_rejected_entities() {
        let mut graph = Graph::default();

        for token in [
            key(ZID, PUBLISHER, CHATTER, FIRST_DEPTH),
            key(OTHER_ZID, PUBLISHER, CHATTER, SECOND_DEPTH),
            key(ZID, SERVICE_SERVER, ADD_TWO_INTS, THIRD_DEPTH),
            key(OTHER_ZID, SERVICE_SERVER, ADD_TWO_INTS, FOURTH_DEPTH),
        ] {
            assert_eq!(graph.put(&token), Ok(true));
        }

        assert_eq!(
            depths(graph.topics(|token| token.zid != ZID)),
            listed(&[(CHATTER, &[SECOND_DEPTH])])
        );
        assert_eq!(
            depths(graph.services(|token| token.zid != ZID)),
            listed(&[(ADD_TWO_INTS, &[FOURTH_DEPTH])])
        );
    }

    #[test]
    fn topic_lists_publishers_before_subscriptions_in_key_order() {
        let mut graph = Graph::default();

        for token in [
            key(OTHER_ZID, SUBSCRIPTION, CHATTER, FIRST_DEPTH),
            key(ZID, PUBLISHER, CHATTER, SECOND_DEPTH),
            key(OTHER_ZID, PUBLISHER, CHATTER, THIRD_DEPTH),
        ] {
            assert_eq!(graph.put(&token), Ok(true));
        }

        assert_eq!(
            depths(graph.topics(|_| true)),
            listed(&[(CHATTER, &[THIRD_DEPTH, SECOND_DEPTH, FIRST_DEPTH])])
        );
    }

    #[test]
    fn service_lists_servers_before_clients_in_key_order() {
        let mut graph = Graph::default();

        for token in [
            key(OTHER_ZID, SERVICE_CLIENT, ADD_TWO_INTS, FIRST_DEPTH),
            key(ZID, SERVICE_SERVER, ADD_TWO_INTS, SECOND_DEPTH),
            key(OTHER_ZID, SERVICE_SERVER, ADD_TWO_INTS, THIRD_DEPTH),
        ] {
            assert_eq!(graph.put(&token), Ok(true));
        }

        assert_eq!(
            depths(graph.services(|_| true)),
            listed(&[(ADD_TWO_INTS, &[THIRD_DEPTH, SECOND_DEPTH, FIRST_DEPTH])])
        );
    }
}
