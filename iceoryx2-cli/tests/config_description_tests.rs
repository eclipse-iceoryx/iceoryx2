// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

mod config_descriptions_tests {
    use std::collections::HashSet;

    use ron::de::from_str;
    use ron::ser::to_string;
    use ron::Value;

    use iceoryx2::config::Config;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cli::config_descriptions::get_sections;

    // Recursively walk through ron::Value to flatten all keys into a HashSet
    fn collect_keys_ron(value: &Value, prefix: String, keys: &mut HashSet<String>) {
        match value {
            Value::Map(map) => {
                for (k, v) in map.iter() {
                    let k_str = match k {
                        Value::String(s) => s.clone(),
                        _ => continue, // skip non-string keys
                    };

                    let new_prefix = if prefix.is_empty() {
                        k_str
                    } else {
                        format!("{}.{}", prefix, k_str)
                    };

                    collect_keys_ron(v, new_prefix, keys);
                }
            }
            _ => {
                keys.insert(prefix);
            }
        }
    }

    #[test]
    fn check_config_description_is_present_for_all_config_parameters() {
        let config = Config::default();

        let ron_string = to_string(&config).expect("Failed to serialize config to RON");
        let parsed: Value = from_str(&ron_string).expect("Invalid RON");

        let mut ron_keys = HashSet::new();
        collect_keys_ron(&parsed, "".to_string(), &mut ron_keys);

        let cli_keys: HashSet<String> = get_sections()
            .iter()
            .flat_map(|section| section.entries.iter())
            .map(|entry| entry.key.to_string())
            .collect();

        let missing_in_config = cli_keys.difference(&ron_keys).collect::<Vec<_>>();
        let extra_in_config = ron_keys.difference(&cli_keys).collect::<Vec<_>>();
        println!("Missing in config: {:?}", extra_in_config);
        assert_that!(missing_in_config.len(), eq 0);
        assert_that!(extra_in_config.len(), eq 0);
    }
}
