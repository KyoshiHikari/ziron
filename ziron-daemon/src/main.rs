//! Ziron Daemon - Background process for aggregating status information

use ziron_core::config::Config;
use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData, ModuleRegistry};
use ziron_core::prompt::PromptRenderer;
use ziron_core::theme::Theme;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::load().unwrap_or_default();
    let mut registry = ModuleRegistry::new();

    // Load modules
    load_modules(&mut registry)?;

    // Load theme
    let theme = if let Some(theme_name) = &config.theme {
        load_theme(theme_name)?
    } else {
        Theme::load_from(&Theme::default_path()?)?
    };

    let renderer = PromptRenderer::new(theme);

    // Start IPC server
    let socket_path = get_socket_path()?;
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    tracing::info!("Daemon started, listening on {:?}", socket_path);

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let registry_clone = registry.clone();
                let renderer_clone = renderer.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(&mut stream, &registry_clone, &renderer_clone).await {
                        tracing::error!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                tracing::error!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_client(
    stream: &mut tokio::net::UnixStream,
    registry: &ModuleRegistry,
    renderer: &PromptRenderer,
) -> Result<()> {
    let mut buffer = vec![0u8; 1024];
    let n = stream.read(&mut buffer).await?;

    if n == 0 {
        return Ok(());
    }

    // Parse request (simplified - in production use proper IPC protocol)
    let context = ModuleContext::from_env()?;
    let mut module_data = Vec::new();

    // Fetch data from all registered modules
    for module_name in registry.all().keys() {
        if let Some(data) = fetch_module_data(module_name, &context, registry).await? {
            module_data.push(data);
        }
    }

    // Render prompt
    let prompt = renderer.render(&context, &module_data)?;

    // Send response
    stream.write_all(prompt.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}

async fn fetch_module_data(
    module_name: &str,
    _context: &ModuleContext,
    _registry: &ModuleRegistry,
) -> Result<Option<ModuleData>> {
    // In a real implementation, this would call the actual module
    // For now, return placeholder data
    Ok(Some(ModuleData {
        module: module_name.to_string(),
        data: serde_json::json!({
            "text": format!("[{}]", module_name)
        }),
        cached: false,
    }))
}

fn load_modules(_registry: &mut ModuleRegistry) -> Result<()> {
    // In a real implementation, scan modules directory
    // For now, just return OK
    Ok(())
}

fn load_theme(name: &str) -> Result<Theme> {
    let theme_path = PathBuf::from("themes").join(name).join("theme.toml");
    Theme::load_from(&theme_path)
}

fn get_socket_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| ziron_core::error::Error::Config("HOME not set".to_string()))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("ziron")
        .join("ziron.sock"))
}

