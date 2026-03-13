#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! toml = "0.8"
//! serde = { version = "1", features = ["derive"] }
//! glob = "0.3"
//! ```

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

use serde::Deserialize;
use std::{collections::HashSet, fs, path::PathBuf};
use toml::Value;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct PropagationAnalysis {
    propagated: HashSet<String>,
    enabled: HashSet<String>,
    disabled: HashSet<String>,
    missing: HashSet<String>,
}

#[derive(Deserialize, Debug)]
struct CargoToml {
    package: Option<Package>,
    workspace: Option<Workspace>,
    features: Option<toml::Table>,
    dependencies: Option<toml::Table>,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    members: Option<Vec<String>>,
}

/// Parses the command-line argument and returns the canonicalized path to the target `Cargo.toml`.
fn parse_manifest_path(args: &[String]) -> Result<PathBuf> {
    if args.len() != 2 {
        eprintln!("Usage: verify_std_propagation <path/to/Cargo.toml>");
        std::process::exit(1);
    }

    Ok(PathBuf::from(&args[1]).canonicalize()?)
}

/// Reads and deserializes a `Cargo.toml` file at the given path.
fn load_manifest(path: &PathBuf) -> Result<CargoToml> {
    let content = fs::read_to_string(path)?;

    Ok(toml::from_str::<CargoToml>(&content)?)
}

/// Walks up the directory tree from the crate manifest's parent to find and load the workspace `Cargo.toml`.
fn load_workspace_manifest(crate_manifest_path: &PathBuf) -> Result<(PathBuf, CargoToml)> {
    let mut current = crate_manifest_path
        .parent()
        .expect("Cargo.toml path has no parent directory")
        .to_path_buf();

    loop {
        let candidate = current.join("Cargo.toml");
        if candidate.exists() {
            if let Ok(cargo_toml) = load_manifest(&candidate) {
                if cargo_toml.workspace.is_some() {
                    return Ok((current, cargo_toml));
                }
            }
        }
        if !current.pop() {
            return Err("Could not find crate workspace manifest".into());
        }
    }
}

/// Locates the `Cargo.toml` for a named crate by checking workspace dependency paths or scanning workspace members.
fn find_crate_manifest_path(
    workspace_manifest_path: &PathBuf,
    workspace_manifest: &CargoToml,
    crate_name: &str,
) -> Option<PathBuf> {
    // Check if dependency specifies a path
    if let Some(dependencies) = &workspace_manifest.dependencies {
        if let Some(Value::Table(workspace_dependency)) = dependencies.get(crate_name) {
            if let Some(Value::String(rel_path)) = workspace_dependency.get("path") {
                let candidate = workspace_manifest_path.join(rel_path).join("Cargo.toml");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    // Otherwise, retrieve path from workspace members
    let workspace_members = workspace_manifest
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.members.as_ref())
        .map(|member| member.clone())
        .unwrap_or_default();

    for member in workspace_members {
        let paths: Vec<PathBuf> = if member.contains('*') {
            let pattern = workspace_manifest_path.join(&member).join("Cargo.toml");
            glob::glob(pattern.to_str()?)
                .ok()?
                .filter_map(|result| result.ok())
                .collect()
        } else {
            vec![workspace_manifest_path.join(&member).join("Cargo.toml")]
        };

        for path in paths {
            if let Ok(data) = load_manifest(&path) {
                if data.package.as_ref().map(|package| package.name.as_str()) == Some(crate_name) {
                    return Some(path.to_path_buf());
                }
            }
        }
    }

    None
}

/// Returns true if the crate corresponding to `dependency_name` defines the `std` feature.
fn dependency_defines_std_feature(
    workspace_manifest_path: &PathBuf,
    workspace_manifest: &CargoToml,
    dependency_name: &str,
) -> bool {
    let Some(dependency_manifest_path) =
        find_crate_manifest_path(workspace_manifest_path, workspace_manifest, dependency_name)
    else {
        // Ignore crates not in workspace
        return false;
    };

    let Ok(dependency_manifest) = load_manifest(&dependency_manifest_path) else {
        // Ignore corrupted manifests
        return false;
    };

    dependency_manifest
        .features
        .is_some_and(|features| features.contains_key("std"))
}

/// Returns true if the crate propagates the `std` feature to `dependency_name` via its own feature
/// definition.
fn propagates_std_feature(crate_manifest: &CargoToml, dependency_name: &str) -> bool {
    crate_manifest
        .features
        .as_ref()
        .and_then(|features| features.get("std"))
        .and_then(|value| value.as_array())
        .is_some_and(|array| {
            array
                .iter()
                .any(|v| v.as_str() == Some(&format!("{dependency_name}/std")))
        })
}

/// Returns true if the dependency declaration enables std via `features = ["std"]`, bypassing
/// feature propagation.
fn enables_std_feature(dependency_value: &Value) -> bool {
    if dependency_value
        .get("features")
        .and_then(|features_value: &Value| features_value.as_array())
        .is_some_and(|array| {
            array
                .iter()
                .any(|feature_value: &Value| feature_value.as_str() == Some("std"))
        })
    {
        return true;
    }

    false
}

/// Returns true if the dependency explicitly sets `features` without including `std`, indicating
/// `std` is intentionally not enabled.
fn disables_std_feature(dependency_value: &Value) -> bool {
    if dependency_value
        .get("features")
        .and_then(|features_value: &Value| features_value.as_array())
        .is_some_and(|array| {
            !array
                .iter()
                .any(|feature_value: &Value| feature_value.as_str() == Some("std"))
        })
    {
        return true;
    }

    false
}

/// Classifies each workspace dependency of the crate by how it handles the `std` feature:
/// propagated via feature flag, explicitly enabled, explicitly disabled, or missing.
fn analyse_propagation_of_std_feature(
    workspace_manifest_path: &PathBuf,
    workspace_manifest: &CargoToml,
    crate_manifest: &CargoToml,
) -> PropagationAnalysis {
    let mut result = PropagationAnalysis {
        propagated: HashSet::new(),
        enabled: HashSet::new(),
        disabled: HashSet::new(),
        missing: HashSet::new(),
    };

    for (dependency_name, dependency_value) in crate_manifest.dependencies.iter().flatten() {
        if !dependency_defines_std_feature(
            workspace_manifest_path,
            workspace_manifest,
            dependency_name,
        ) {
            continue;
        }

        if propagates_std_feature(crate_manifest, dependency_name) {
            result.propagated.insert(dependency_name.to_string());
        } else if enables_std_feature(dependency_value) {
            result.enabled.insert(dependency_name.to_string());
        } else if disables_std_feature(dependency_value) {
            result.disabled.insert(dependency_name.to_string());
        } else {
            result.missing.insert(dependency_name.to_string());
        }
    }

    result
}

/// Prints a sorted list of dependencies under a given label.
///
/// Example output:
/// ```
/// propagated:
///   iceoryx2-log
///   iceoryx2-bb-print
/// ```
fn print_dependency_notice(label: &str, dependencies: &HashSet<String>) {
    println!("{label}:");
    let mut sorted: Vec<&String> = dependencies.iter().collect();
    sorted.sort();
    for dependency in sorted {
        println!("  {dependency}");
    }
    println!();
}

/// Prints a suggested `std = [...]` block containing already-propagated dependencies and those
/// that are missing propagation (annotated with `# missing`).
///
/// Example output:
/// ```
/// ✗ std not propagated to all dependencies
///
/// std = [
///   "iceoryx2-bb-posix/std",
///   "iceoryx2-log/std",  # missing
/// ]
/// ```
fn print_missing_std_propagations(propagated: &HashSet<String>, missing: &HashSet<String>) {
    println!("✗ std not propagated to all dependencies");
    println!();

    let mut entries: Vec<(String, bool)> = propagated
        .iter()
        .map(|dependency| (format!("{dependency}/std"), false))
        .chain(
            missing
                .iter()
                .map(|dependency| (format!("{dependency}/std"), true)),
        )
        .collect();

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    println!("std = [");
    for (entry, is_missing) in &entries {
        if *is_missing {
            println!("  \"{entry}\",  # missing");
        } else {
            println!("  \"{entry}\",");
        }
    }
    println!("]");
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let crate_manifest_path = parse_manifest_path(&args)?;
    let crate_manifest = load_manifest(&crate_manifest_path)?;
    let (workspace_manifest_path, workspace_manifest) =
        load_workspace_manifest(&crate_manifest_path)
            .expect("Could not load workspace of {crate_manifest}");

    let analysis = analyse_propagation_of_std_feature(
        &workspace_manifest_path,
        &workspace_manifest,
        &crate_manifest,
    );

    if !analysis.missing.is_empty() {
        print_missing_std_propagations(&analysis.propagated, &analysis.missing);
        std::process::exit(1);
    }

    if !analysis.propagated.is_empty() {
        print_dependency_notice("propagated", &analysis.propagated);
    }

    if !analysis.enabled.is_empty() {
        print_dependency_notice("enabled", &analysis.enabled);
    }

    if !analysis.disabled.is_empty() {
        print_dependency_notice("disabled", &analysis.disabled);
    }

    println!("✓ check passed");

    Ok(())
}
