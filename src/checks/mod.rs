pub mod existence;
pub mod typosquat;
pub mod signals;
pub mod vulnerability;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum RiskLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub level: RiskLevel,
    pub check: String,
    pub description: String,
}
