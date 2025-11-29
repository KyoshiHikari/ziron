//! Docker module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::process::Command;

/// Docker module implementation
pub struct DockerModule;

impl DockerModule {
    /// Fetch Docker context information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let docker_info = Self::get_docker_info(&context.current_dir)?;

        if let Some(info) = docker_info {
            let mut parts = vec![];
            
            if let Some(ref context) = info.context {
                parts.push(context.clone());
            }
            
            if info.container_count > 0 {
                parts.push(format!("{}", info.container_count));
            }
            
            if let Some(ref project) = info.compose_project {
                parts.push(format!("compose:{}", project));
            }

            Ok(ModuleData {
                module: "docker".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "context": info.context,
                    "container_count": info.container_count,
                    "compose_project": info.compose_project,
                    "has_dockerfile": info.has_dockerfile,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "docker".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "context": null,
                }),
                cached: false,
            })
        }
    }

    fn get_docker_info(path: &PathBuf) -> Result<Option<DockerInfo>> {
        // Check for docker-compose.yml
        let compose_file = path.join("docker-compose.yml");
        let compose_project = if compose_file.exists() {
            // Try to get project name from environment or use directory name
            std::env::var("COMPOSE_PROJECT_NAME")
                .ok()
                .or_else(|| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                })
        } else {
            None
        };

        // Check for Dockerfile
        let has_dockerfile = path.join("Dockerfile").exists();

        // Get Docker context
        let context_output = Command::new("docker")
            .args(&["context", "show"])
            .output();

        let context = context_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let ctx = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if ctx != "default" {
                        Some(ctx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        // Get running container count
        let container_output = Command::new("docker")
            .args(&["ps", "-q"])
            .output();

        let container_count = container_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).lines().count())
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Only return info if we have something meaningful
        if context.is_some() || container_count > 0 || compose_project.is_some() || has_dockerfile {
            Ok(Some(DockerInfo {
                context,
                container_count,
                compose_project,
                has_dockerfile,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
struct DockerInfo {
    context: Option<String>,
    container_count: usize,
    compose_project: Option<String>,
    has_dockerfile: bool,
}

