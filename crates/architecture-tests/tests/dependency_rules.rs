//! Dependency rules between crates (SPEC §14.5, AGENTS.md "Code rules").
//!
//! A rule names a workspace crate and the complete set of other workspace crates it may
//! depend on, plus external crates it must never pull in. Changing a rule needs an ADR.

use std::collections::BTreeSet;

use cargo_metadata::{DependencyKind, MetadataCommand, Package};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Rule {
    krate: &'static str,
    may_use_workspace: &'static [&'static str],
    must_not_use_external: &'static [&'static str],
}

/// Frameworks and runtimes that must stay out of the pure and in-REAPER crates.
const HEAVY: &[&str] = &["tauri", "tokio", "reqwest", "axum", "hyper", "async-std"];

const RULES: &[Rule] = &[
    Rule {
        krate: "shared-kernel",
        may_use_workspace: &[],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "performance",
        may_use_workspace: &["shared-kernel"],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "catalogue",
        may_use_workspace: &["shared-kernel"],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "setlists",
        may_use_workspace: &["shared-kernel"],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "projections",
        may_use_workspace: &["shared-kernel", "performance", "catalogue", "setlists"],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "protocol",
        may_use_workspace: &["shared-kernel"],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "link",
        may_use_workspace: &["shared-kernel", "protocol"],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "reaper-port",
        may_use_workspace: &["shared-kernel", "performance"],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "reaper-extension",
        may_use_workspace: &[
            "shared-kernel",
            "performance",
            "catalogue",
            "setlists",
            "projections",
            "protocol",
            "link",
            "reaper-port",
        ],
        must_not_use_external: HEAVY,
    },
    Rule {
        krate: "app",
        may_use_workspace: &["shared-kernel", "protocol", "link"],
        must_not_use_external: &[],
    },
];

fn normal_dependency_names(package: &Package) -> BTreeSet<String> {
    package
        .dependencies
        .iter()
        .filter(|d| d.kind == DependencyKind::Normal)
        .map(|d| d.name.clone())
        .collect()
}

#[test]
fn crates_only_depend_on_what_the_architecture_allows() -> TestResult {
    let metadata = MetadataCommand::new().exec()?;
    let workspace: BTreeSet<String> = metadata
        .workspace_packages()
        .iter()
        .map(|p| p.name.to_string())
        .collect();

    for rule in RULES {
        let package = metadata
            .workspace_packages()
            .into_iter()
            .find(|p| p.name.as_str() == rule.krate)
            .ok_or_else(|| format!("rule names unknown crate `{}`", rule.krate))?;
        let deps = normal_dependency_names(package);

        for dep in deps.iter().filter(|d| workspace.contains(*d)) {
            assert!(
                rule.may_use_workspace.contains(&dep.as_str()),
                "`{}` must not depend on workspace crate `{dep}`",
                rule.krate
            );
        }
        for forbidden in rule.must_not_use_external {
            assert!(
                !deps.contains(*forbidden),
                "`{}` must not depend on `{forbidden}`",
                rule.krate
            );
        }
    }
    Ok(())
}

#[test]
fn nothing_depends_on_the_extension() -> TestResult {
    let metadata = MetadataCommand::new().exec()?;
    for package in metadata.workspace_packages() {
        let deps = normal_dependency_names(package);
        assert!(
            !deps.contains("reaper-extension"),
            "`{}` must not depend on `reaper-extension`; it is loaded by REAPER, never linked",
            package.name
        );
    }
    Ok(())
}

#[test]
fn every_rule_names_an_existing_crate() -> TestResult {
    let metadata = MetadataCommand::new().exec()?;
    let workspace: BTreeSet<String> = metadata
        .workspace_packages()
        .iter()
        .map(|p| p.name.to_string())
        .collect();
    for rule in RULES {
        assert!(
            workspace.contains(rule.krate),
            "unknown crate `{}` in RULES",
            rule.krate
        );
        for allowed in rule.may_use_workspace {
            assert!(
                workspace.contains(*allowed),
                "unknown allowed crate `{allowed}`"
            );
        }
    }
    Ok(())
}
