//! Core library for Reflections - Solidity analysis and reflection tool
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use errors::ReflectionsError;

pub type Result<T> = std::result::Result<T, ReflectionsError>;

pub mod config;
pub mod errors;
pub mod utils;
