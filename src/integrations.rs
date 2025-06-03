use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

/// Contains information for all things related to programmability and
/// external integrations in Basalt.
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Integrations {
    /// Paths to files that will be executed on server events
    #[serde_as(as = "OneOrMany<_>")]
    pub events: Vec<PathBuf>,
}
