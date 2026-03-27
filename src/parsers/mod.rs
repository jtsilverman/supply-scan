pub mod npm;
pub mod pypi;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub ecosystem: Ecosystem,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Ecosystem {
    Npm,
    PyPI,
}
