//! Check this package with the debtmap dev-dependency.

use std::path::PathBuf;

use debtmap::commands::validate::{ValidateConfig, validate_project};

#[test]
fn debt_density_stays_within_the_configured_limit() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    validate_project(ValidateConfig {
        path: root.clone(),
        config: Some(root.join(".debtmap.toml")),
        coverage_file: None,
        format: None,
        output: None,
        enable_context: false,
        context_providers: None,
        disable_context: None,
        max_debt_density: None,
        top: None,
        tail: None,
        semantic_off: false,
        verbosity: 0,
        no_parallel: false,
        jobs: 0,
        show_splits: false,
    })
    .expect("debtmap validation failed");
}
