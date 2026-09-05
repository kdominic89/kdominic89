//! Typed public configuration, deterministic graph construction, and validation for SOURCEFIELD.
#![deny(missing_docs)]

mod config;
mod graph;
mod model;
mod validate;

pub use config::{ConfigError, load_config};
pub use graph::{GraphError, build_state, state_is_live};
pub use model::*;
pub use validate::{ValidationError, validate_config, validate_state};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_mode_defaults_to_preview() {
        assert_eq!(Snapshot::default().mode, SnapshotMode::Preview);
    }

    #[test]
    fn private_abstract_serialization_is_stable() {
        let value = serde_json::to_string(&Visibility::PrivateAbstract).unwrap();
        assert_eq!(value, r#""private-abstract""#);
    }
}

#[cfg(test)]
mod contract_tests;
