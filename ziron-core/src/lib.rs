//! Ziron Core Library
//!
//! This library provides the core functionality for the Ziron shell framework,
//! including configuration loading, module registry, event system, prompt pipeline,
//! and IPC interface.

pub mod cache;
pub mod config;
pub mod error;
pub mod event;
pub mod ipc;
pub mod module;
pub mod prompt;
pub mod theme;

pub use error::{Error, Result};

