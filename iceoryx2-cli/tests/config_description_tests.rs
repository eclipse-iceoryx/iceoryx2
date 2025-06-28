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
    use toml::Value;

    use iceoryx2::config::Config;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cli::config_descriptions::get_sections;

    // Recursively walk through toml::Value to flatten all keys into a HashSet
    fn collect_keys(value: &Value, prefix: String, keys: &mut HashSet<String>) {
        match value {
            Value::Table(table) => {
                for (k, v) in table {
                    let new_prefix = if prefix.is_empty() {
                        k.to_string()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    collect_keys(v, new_prefix, keys);
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
        let toml_string = toml::to_string(&config).expect("Failed to serialize config to TOML");
        let sections = get_sections();

        let parsed: Value = toml::from_str(&toml_string).expect("Invalid TOML");
        let mut toml_keys = HashSet::new();
        collect_keys(&parsed, "".to_string(), &mut toml_keys);

        // let mut missing_keys = Vec::new();

        let cli_keys: HashSet<String> = sections
            .iter()
            .flat_map(|section| section.entries.iter())
            .map(|entry| entry.key.to_string())
            .collect();

        // Find missing in TOML
        let missing_in_toml: Vec<_> = cli_keys.difference(&toml_keys).collect();

        // Find extra in TOML
        let extra_in_toml: Vec<_> = toml_keys.difference(&cli_keys).collect();

        assert_that!(missing_in_toml.len(), eq 0);
        assert_that!(extra_in_toml.len(), eq 0);
    }
}
