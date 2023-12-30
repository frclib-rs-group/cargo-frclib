use std::path::PathBuf;

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// The default team number to use when creating new projects.
    pub team: Option<u16>,
    /// An array of paths to deploy descriptors or directories containing deploy descriptors.
    pub deploy_descriptor_paths: Vec<PathBuf>,
    
}