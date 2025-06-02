use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Contains information for all things related to programmability and
/// external integrations in Basalt.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Integrations {
    /// Paths to files that will be executed on server events
    pub events: Vec<PathBuf>,
}
