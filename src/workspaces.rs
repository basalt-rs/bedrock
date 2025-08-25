use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::roi::RawOrImport;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Workspace {
    /// Optional directory on which to base the generation of this workspace
    from: Option<PathBuf>,
    /// Script executed to generate the workspace
    setup: Option<RawOrImport<String>>,
    /// Script executed to install any required dependencies if any are required
    install: Option<RawOrImport<String>>,
}
