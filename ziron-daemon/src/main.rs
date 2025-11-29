//! Ziron Daemon - Background process for aggregating status information

mod daemon;
mod watchers;

use ziron_core::cache::Cache;
use ziron_core::config::Config;
use ziron_core::error::Result;
use ziron_core::ipc::{Message, MessagePayload, Request, Response};
use ziron_core::module::{ModuleContext, ModuleData, ModuleRegistry};
use ziron_core::prompt::PromptRenderer;
use ziron_core::theme::Theme;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "stop" => {
                if let Some(pid) = daemon::get_daemon_pid() {
                    // Use std::process::Command to send SIGTERM
                    use std::process::Command;
                    let output = Command::new("kill")
                        .arg("-TERM")
                        .arg(pid.to_string())
                        .output();
                    match output {
                        Ok(_) => println!("Daemon stopped (PID: {})", pid),
                        Err(e) => eprintln!("Failed to stop daemon (PID: {}): {}", pid, e),
                    }
                } else {
                    eprintln!("Daemon is not running");
                }
                return Ok(());
            }
            "status" => {
                if daemon::is_running() {
                    if let Some(pid) = daemon::get_daemon_pid() {
                        println!("Daemon is running (PID: {})", pid);
                    } else {
                        println!("Daemon is running");
                    }
                } else {
                    println!("Daemon is not running");
                }
                return Ok(());
            }
            _ => {}
        }
    }

    // Check if daemon is already running
    if daemon::is_running() {
        eprintln!("Daemon is already running");
        return Ok(());
    }

    // Write PID file
    daemon::write_pid_file()?;

    // Setup signal handlers for graceful shutdown
    let mut shutdown = setup_signal_handlers();

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

    // Create cache with TTL from config
    let cache_ttl = Duration::from_millis(config.performance.cache_ttl_ms);
    let cache = Cache::new(cache_ttl, 1000);

    // Setup event system
    let (event_tx, _event_rx) = broadcast::channel(100);
    
    // Start file system watcher for current directory
    let mut watcher_manager = watchers::WatcherManager::new(cache.clone(), event_tx.clone());
    if let Ok(current_dir) = std::env::current_dir() {
        if let Err(e) = watcher_manager.watch_directory(&current_dir) {
            tracing::warn!("Failed to start file watcher: {}", e);
        } else {
            tracing::info!("File system watcher started for {:?}", current_dir);
        }
        
        // Also watch Git repository if present
        if watchers::WatcherManager::is_git_repo(&current_dir) {
            if let Err(e) = watcher_manager.watch_git_repo(&current_dir) {
                tracing::warn!("Failed to watch Git repository: {}", e);
            } else {
                tracing::info!("Git repository watcher started");
            }
        }
    }

    // Start IPC server
    let socket_path = get_socket_path()?;
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    tracing::info!("Daemon started, listening on {:?}", socket_path);

    // Request ID counter
    let request_id_counter = AtomicU64::new(0);

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                tracing::info!("Shutdown signal received");
                daemon::remove_pid_file()?;
                break;
            }
            result = listener.accept() => {
                match result {
                    Ok((mut stream, _)) => {
                        let registry_clone = registry.clone();
                        let renderer_clone = renderer.clone();
                        let cache_clone = cache.clone();
                        let config_clone = config.clone();
                        let request_id = request_id_counter.fetch_add(1, Ordering::Relaxed);
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(&mut stream, &registry_clone, &renderer_clone, &cache_clone, &config_clone, request_id).await {
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
    }

    Ok(())
}

fn setup_signal_handlers() -> tokio::sync::broadcast::Receiver<()> {
    let (tx, rx) = tokio::sync::broadcast::channel(1);
    
    let tx1 = tx.clone();
    tokio::spawn(async move {
        use signal_hook::consts::signal::SIGTERM;
        use signal_hook::consts::signal::SIGINT;
        use signal_hook_tokio::Signals;
        use futures_util::StreamExt;
        
        let mut signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
        while let Some(signal) = signals.next().await {
            if signal == SIGTERM || signal == SIGINT {
                let _ = tx1.send(());
                break;
            }
        }
    });
    
    rx
}

async fn handle_client(
    stream: &mut tokio::net::UnixStream,
    registry: &ModuleRegistry,
    renderer: &PromptRenderer,
    cache: &Cache,
    config: &Config,
    request_id: u64,
) -> Result<()> {
    // Read message length (4 bytes)
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_le_bytes(len_buf) as usize;

    // Read message data
    let mut buffer = vec![0u8; msg_len];
    stream.read_exact(&mut buffer).await?;

    // Deserialize message
    let message = Message::deserialize(&buffer)
        .map_err(|e| ziron_core::error::Error::Config(format!("Failed to deserialize message: {}", e)))?;

    // Handle request
    let response = match &message.payload {
        MessagePayload::Request(request) => {
            match request {
                Request::GetPrompt { context } => {
                    let mut module_data = Vec::new();
                    
                    // Get modules from theme segments (what should be displayed)
                    // Fallback to config.modules if theme has no segments
                    let modules_to_fetch: Vec<String> = if !renderer.theme().segments.is_empty() {
                        renderer.theme().segments.iter()
                            .map(|s| s.module.clone())
                            .collect()
                    } else {
                        config.modules.clone()
                    };
                    
                    // Fetch data from modules (with caching)
                    for module_name in &modules_to_fetch {
                        let cache_key = format!("{}:{}", module_name, context.current_dir.display());
                        
                        // Try to get from cache first
                        if let Some(cached_data) = cache.get(&cache_key) {
                            module_data.push(cached_data);
                        } else {
                            // Fetch fresh data
                            if let Some(data) = fetch_module_data(module_name, context, registry).await? {
                                // Store in cache
                                cache.set(cache_key, data.clone());
                                module_data.push(data);
                            }
                        }
                    }
                    // Render prompt
                    let prompt = renderer.render(context, &module_data)?;
                    Response::Prompt(prompt)
                }
                Request::GetModuleData { module, context } => {
                    let cache_key = format!("{}:{}", module, context.current_dir.display());
                    
                    // Try cache first
                    if let Some(cached_data) = cache.get(&cache_key) {
                        Response::ModuleData(cached_data)
                    } else if let Some(data) = fetch_module_data(module, context, registry).await? {
                        // Store in cache
                        cache.set(cache_key, data.clone());
                        Response::ModuleData(data)
                    } else {
                        Response::Error(format!("Module {} not found", module))
                    }
                }
                Request::InvalidateCache { module } => {
                    cache.invalidate(module.as_deref());
                    Response::Ok
                }
                Request::GetCacheStats => {
                    let stats = cache.stats();
                    Response::CacheStats {
                        hits: stats.hits,
                        misses: stats.misses,
                        size: stats.size,
                    }
                }
                Request::Shutdown => {
                    // Send response before shutting down
                    let response_msg = Message::new_response(request_id, Response::Ok);
                    let response_data = response_msg.serialize()
                        .map_err(|e| ziron_core::error::Error::Config(format!("Failed to serialize response: {}", e)))?;
                    let len = response_data.len() as u32;
                    stream.write_all(&len.to_le_bytes()).await?;
                    stream.write_all(&response_data).await?;
                    stream.flush().await?;
                    
                    // Trigger shutdown
                    daemon::remove_pid_file()?;
                    std::process::exit(0);
                }
                Request::HealthCheck => {
                    Response::Health {
                        status: "ok".to_string(),
                        uptime: 0, // TODO: Track uptime
                    }
                }
            }
        }
        MessagePayload::Response(_) => {
            Response::Error("Received response instead of request".to_string())
        }
    };

    // Send response
    let response_msg = Message::new_response(request_id, response);
    let response_data = response_msg.serialize()
        .map_err(|e| ziron_core::error::Error::Config(format!("Failed to serialize response: {}", e)))?;
    let len = response_data.len() as u32;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(&response_data).await?;
    stream.flush().await?;

    Ok(())
}

async fn fetch_module_data(
    module_name: &str,
    context: &ModuleContext,
    _registry: &ModuleRegistry,
) -> Result<Option<ModuleData>> {
    // Call the actual module implementation
    // Since Rust is statically compiled, we use a match statement
    let result = match module_name {
        "git" => ziron_module_git::GitModule::fetch_data(context),
        "sysinfo" => ziron_module_sysinfo::SysInfoModule::fetch_data(context),
        "exitcode" => exitcode::ExitCodeModule::fetch_data(context),
        "timer" => timer::TimerModule::fetch_data(context),
        "time" => time::TimeModule::fetch_data(context),
        "venv" => venv::VenvModule::fetch_data(context),
        "node" => node::NodeModule::fetch_data(context),
        "rust" => rust::RustModule::fetch_data(context),
        "conda" => conda::CondaModule::fetch_data(context),
        "svn" => ziron_module_svn::SvnModule::fetch_data(context),
        "mercurial" => ziron_module_mercurial::MercurialModule::fetch_data(context),
        "docker" => ziron_module_docker::DockerModule::fetch_data(context),
        "kubernetes" => ziron_module_kubernetes::KubernetesModule::fetch_data(context),
        "aws" => ziron_module_aws::AwsModule::fetch_data(context),
        "gcp" => ziron_module_gcp::GcpModule::fetch_data(context),
        "azure" => ziron_module_azure::AzureModule::fetch_data(context),
        "terraform" => ziron_module_terraform::TerraformModule::fetch_data(context),
        "go" => ziron_module_go::GoModule::fetch_data(context),
        _ => {
            // Unknown module, return None
            return Ok(None);
        }
    };
    
    match result {
        Ok(data) => Ok(Some(data)),
        Err(e) => {
            tracing::warn!("Error fetching data for module {}: {}", module_name, e);
            Ok(None)
        }
    }
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

