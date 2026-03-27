pub mod npm;
pub mod osv;
pub mod pypi;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct PackageMetadata {
    pub exists: bool,
    pub first_published: Option<String>,
    pub latest_published: Option<String>,
    pub maintainer_count: usize,
    pub has_install_scripts: bool,
    pub install_scripts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub summary: String,
    pub severity: Option<String>,
}
