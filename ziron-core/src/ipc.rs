//! IPC protocol for daemon communication

use serde::{Deserialize, Serialize};
use crate::module::{ModuleContext, ModuleData};

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Request message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    /// Request prompt rendering
    GetPrompt {
        context: ModuleContext,
    },
    /// Request module data
    GetModuleData {
        module: String,
        context: ModuleContext,
    },
    /// Invalidate cache for a module or all modules
    InvalidateCache {
        module: Option<String>,
    },
    /// Get cache statistics
    GetCacheStats,
    /// Shutdown daemon
    Shutdown,
    /// Health check
    HealthCheck,
}

/// Response message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Prompt string
    Prompt(String),
    /// Module data
    ModuleData(ModuleData),
    /// Cache statistics
    CacheStats {
        hits: u64,
        misses: u64,
        size: usize,
    },
    /// Success response
    Ok,
    /// Error response
    Error(String),
    /// Health check response
    Health {
        status: String,
        uptime: u64,
    },
}

/// IPC message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub version: u32,
    pub request_id: u64,
    pub payload: MessagePayload,
}

/// Message payload (request or response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Request(Request),
    Response(Response),
}

impl Message {
    /// Create a new request message
    pub fn new_request(request_id: u64, request: Request) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            request_id,
            payload: MessagePayload::Request(request),
        }
    }

    /// Create a new response message
    pub fn new_response(request_id: u64, response: Response) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            request_id,
            payload: MessagePayload::Response(response),
        }
    }

    /// Serialize message to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

