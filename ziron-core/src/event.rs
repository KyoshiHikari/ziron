//! Event system for module hooks

use serde::{Deserialize, Serialize};

/// Event types that modules can subscribe to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Before prompt is rendered
    PrePrompt,
    /// After prompt is rendered
    PostPrompt,
    /// Directory changed
    DirectoryChange,
    /// Command executed
    CommandExecuted,
    /// Shell initialized
    ShellInit,
}

/// Event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub data: serde_json::Value,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, data: serde_json::Value) -> Self {
        Self { event_type, data }
    }

    /// Create a pre-prompt event
    pub fn pre_prompt(data: serde_json::Value) -> Self {
        Self::new(EventType::PrePrompt, data)
    }

    /// Create a post-prompt event
    pub fn post_prompt(data: serde_json::Value) -> Self {
        Self::new(EventType::PostPrompt, data)
    }

    /// Create a directory change event
    pub fn directory_change(path: String) -> Self {
        Self::new(
            EventType::DirectoryChange,
            serde_json::json!({ "path": path }),
        )
    }
}

