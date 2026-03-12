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
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use toml::Value;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

struct WorkspaceContext {
    root: PathBuf,
    dependencies: toml::Table,
}

struct Dependency<'a> {
    name: &'a str,
    value: &'a Value,
}

/// Parses the command-line argument and returns the canonicalized path to the target `Cargo.toml`.
fn parse_crate_toml_path(args: &[String]) -> Result<PathBuf> {
    if args.len() != 2 {
        eprintln!("Usage: verify_std_propagation <path/to/Cargo.toml>");
        std::process::exit(1);
    }

    Ok(PathBuf::from(&args[1]).canonicalize()?)
}

/// Reads and deserializes a `Cargo.toml` file at the given path.
fn load_toml(path: &Path) -> Result<CargoToml> {
    let content = fs::read_to_string(path)?;

    Ok(toml::from_str::<CargoToml>(&content)?)
}

/// Walks up the directory tree from `start` to find the root directory containing a workspace `Cargo.toml`.
fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let candidate = current.join("Cargo.toml");
        if candidate.exists() {
            if let Ok(content) = fs::read_to_string(&candidate) {
                if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                    if value.get("workspace").is_some() {
                        return Some(current);
                    }
                }
            }
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Locates the workspace root from the given crate directory and loads the workspace-level dependency table.
fn load_workspace_context(crate_directory: &Path) -> Result<WorkspaceContext> {
    let workspace_root =
        find_workspace_root(crate_directory).expect("Could not find workspace root");

    let content = fs::read_to_string(workspace_root.join("Cargo.toml"))?;
    let value = toml::from_str::<toml::Value>(&content)?;
    let workspace_dependencies = match value
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.as_table())
        .cloned()
    {
        Some(dependencies) => dependencies,
        None => toml::Table::new(),
    };

    Ok(WorkspaceContext {
        root: workspace_root,
        dependencies: workspace_dependencies,
    })
}

/// Locates the `Cargo.toml` for a named crate by checking workspace dependency paths then scanning workspace members.
fn find_crate_toml(context: &WorkspaceContext, crate_name: &str) -> Option<PathBuf> {
    // Try path from workspace.dependencies first
    if let Some(Value::Table(workspace_dependency)) = context.dependencies.get(crate_name) {
        if let Some(Value::String(rel_path)) = workspace_dependency.get("path") {
            let candidate = context.root.join(rel_path).join("Cargo.toml");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // Fallback: scan workspace members
    let workspace_toml = load_toml(&context.root.join("Cargo.toml")).ok()?;
    let members = match workspace_toml.workspace?.members {
        Some(members) => members,
        None => vec![],
    };

    for member in members {
        let paths: Vec<PathBuf> = if member.contains('*') {
            let pattern = context.root.join(&member).join("Cargo.toml");
            glob::glob(pattern.to_str()?)
                .ok()?
                .filter_map(|result| result.ok())
                .collect()
        } else {
            vec![context.root.join(&member).join("Cargo.toml")]
        };

        for path in paths {
            if let Ok(data) = load_toml(&path) {
                if data.package.as_ref().map(|package| package.name.as_str()) == Some(crate_name) {
                    return Some(path.to_path_buf());
                }
            }
        }
    }

    None
}

/// Returns true if the crate corresponding to `dependency` declares a `std` feature.
fn has_std_feature(context: &WorkspaceContext, dependency: &Dependency) -> bool {
    let Some(path) = find_crate_toml(context, dependency.name) else {
        return false;
    };
    let Ok(data) = load_toml(&path) else {
        return false;
    };
    data.features
        .is_some_and(|features| features.contains_key("std"))
}

/// Returns true if the dependency declaration hardcodes `features = ["std"]`, bypassing feature propagation.
fn hardcodes_std_feature(context: &WorkspaceContext, dependency: &Dependency) -> bool {
    let hardcodes = |value: &Value| -> bool {
        value
            .get("features")
            .and_then(|features_value: &Value| features_value.as_array())
            .is_some_and(|array| {
                array
                    .iter()
                    .any(|feature_value: &Value| feature_value.as_str() == Some("std"))
            })
    };

    // Check the inline declaration first
    if hardcodes(dependency.value) {
        return true;
    }

    // If it's a workspace dependency, also check the workspace-level declaration
    if dependency
        .value
        .get("workspace")
        .and_then(|value: &Value| value.as_bool())
        == Some(true)
    {
        if let Some(workspace_dependency) = context.dependencies.get(dependency.name) {
            return hardcodes(workspace_dependency);
        }
    }

    false
}

/// Returns true if the dependency explicitly sets `features` without including `std`, indicating `std` is intentionally not enabled.
fn explicitly_omits_std_feature(context: &WorkspaceContext, dependency: &Dependency) -> bool {
    let omits = |value: &Value| -> bool {
        value
            .get("features")
            .and_then(|features_value: &Value| features_value.as_array())
            .is_some_and(|array| {
                !array
                    .iter()
                    .any(|feature_value: &Value| feature_value.as_str() == Some("std"))
            })
    };

    if omits(dependency.value) {
        return true;
    }

    // If it's a workspace dependency, also check the workspace-level declaration
    if dependency
        .value
        .get("workspace")
        .and_then(|value: &Value| value.as_bool())
        == Some(true)
    {
        if let Some(workspace_dependency) = context.dependencies.get(dependency.name) {
            return omits(workspace_dependency);
        }
    }

    false
}

/// Collects the names of dependencies already listed under the `std` feature of the crate.
fn collect_already_propagated(crate_features: Option<&toml::Table>) -> HashSet<String> {
    crate_features
        .and_then(|features: &toml::Table| features.get("std"))
        .and_then(|value: &Value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value: &Value| value.as_str())
        .filter_map(|feature_entry: &str| {
            let (dependency, feature) = feature_entry.split_once('/')?;
            (feature == "std").then(|| dependency.to_string())
        })
        .collect()
}

/// Returns the names of dependencies that have a `std` feature but explicitly omit it via `features = []`.
fn collect_explicitly_omitted(
    workspace_context: &WorkspaceContext,
    crate_toml: &CargoToml,
    already_propagated: &HashSet<String>,
) -> HashSet<String> {
    // Can be extended to include dev-dependencies etc.
    let all_dependencies = [&crate_toml.dependencies];

    let mut omitted: HashSet<String> = HashSet::new();
    for dependencies in &all_dependencies {
        if let Some(dependencies) = dependencies {
            for (dependency_name, dependency_value) in dependencies.iter() {
                if already_propagated.contains(dependency_name.as_str()) {
                    continue;
                }
                if hardcodes_std_feature(
                    workspace_context,
                    &Dependency {
                        name: dependency_name,
                        value: dependency_value,
                    },
                ) {
                    continue;
                }
                let dependency = Dependency {
                    name: dependency_name,
                    value: dependency_value,
                };
                if explicitly_omits_std_feature(workspace_context, &dependency)
                    && has_std_feature(workspace_context, &dependency)
                {
                    omitted.insert(dependency_name.to_string());
                }
            }
        }
    }
    omitted
}

/// Prints a notice listing dependencies whose `std` feature is explicitly omitted.
///
/// Example output:
/// ```
/// ~ std explicitly omitted for:
///   iceoryx2-log
///   iceoryx2-bb-print
/// ```
fn print_explicitly_omitted(omitted: &HashSet<String>) {
    println!("~ std explicitly omitted for:");
    let mut sorted: Vec<&String> = omitted.iter().collect();
    sorted.sort();
    for dependency in sorted {
        println!("  {dependency}");
    }
    println!();
}

/// Returns the name and dependency section of each dependency that has a `std` feature but is not propagated by the crate.
fn collect_missing_propagations(
    workspace_context: &WorkspaceContext,
    crate_toml: &CargoToml,
    already_propagated: &HashSet<String>,
) -> Vec<(String, String)> {
    // Can be extended to include dev-dependencies etc.
    let all_dependencies = [("dependencies", &crate_toml.dependencies)];

    let mut missing: Vec<(String, String)> = vec![];
    for (section, dependencies) in &all_dependencies {
        if let Some(dependencies) = dependencies {
            for (dependency_name, dependency_value) in dependencies.iter() {
                if already_propagated.contains(dependency_name.as_str()) {
                    continue;
                }
                let dependency = Dependency {
                    name: dependency_name,
                    value: dependency_value,
                };
                if hardcodes_std_feature(workspace_context, &dependency) {
                    continue;
                }
                if explicitly_omits_std_feature(workspace_context, &dependency) {
                    continue;
                }
                if has_std_feature(workspace_context, &dependency) {
                    missing.push((dependency_name.to_string(), section.to_string()));
                }
            }
        }
    }

    missing
}

/// Prints the current and missing `std` feature entries as a suggested corrected `std = [...]` block.
///
/// Example output:
/// ```
/// ✗ std not propagated to dependencies
///
/// std = [
///   "iceoryx2-log/std",  # missing
///   "iceoryx2-bb-posix/std",
/// ]
/// ```
fn print_missing_std_propagations(
    missing: &[(String, String)],
    crate_features: Option<&toml::Table>,
) {
    println!("✗ std not propagated to dependencies");
    println!();

    println!("std = [");
    let mut current: Vec<String> = crate_features
        .and_then(|features: &toml::Table| features.get("std"))
        .and_then(|value: &Value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value: &Value| value.as_str().map(String::from))
        .chain(
            missing
                .iter()
                .map(|(dependency, _)| format!("{dependency}/std")),
        )
        .collect();

    current.sort();

    let missing_set: HashSet<&str> = missing
        .iter()
        .map(|(dependency, _)| dependency.as_str())
        .collect();

    for entry in &current {
        let dependency = match entry.split_once('/') {
            Some((name, _)) => name,
            None => entry,
        };
        if missing_set.contains(dependency) {
            println!("  \"{entry}\",  # missing");
        } else {
            println!("  \"{entry}\",");
        }
    }
    println!("]");
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let crate_toml_path = parse_crate_toml_path(&args)?;
    let crate_directory = crate_toml_path
        .parent()
        .expect("Cargo.toml path has no parent directory");

    let workspace_context = load_workspace_context(crate_directory)?;
    let crate_toml = load_toml(&crate_toml_path)?;
    let crate_features = crate_toml.features.as_ref();

    let already_propagated = collect_already_propagated(crate_features);
    let omitted = collect_explicitly_omitted(&workspace_context, &crate_toml, &already_propagated);
    let missing =
        collect_missing_propagations(&workspace_context, &crate_toml, &already_propagated);

    if !omitted.is_empty() {
        print_explicitly_omitted(&omitted);
    }

    if missing.is_empty() {
        println!("✓ std properly propagated");
    } else {
        print_missing_std_propagations(&missing, crate_features);
        std::process::exit(1);
    }

    Ok(())
}
