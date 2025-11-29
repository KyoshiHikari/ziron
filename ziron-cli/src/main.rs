//! Ziron CLI - Command-line interface for managing Ziron

use clap::{Parser, Subcommand};
use ziron_core::config::Config;
use ziron_core::error::Result;

#[derive(Parser)]
#[command(name = "ziron")]
#[command(about = "Ziron shell framework CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Ziron configuration
    Init {
        /// Overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Manage themes
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
    /// Validate configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum PluginAction {
    /// Add a plugin
    Add {
        /// Plugin name
        name: String,
    },
    /// Remove a plugin
    Remove {
        /// Plugin name
        name: String,
    },
    /// List installed plugins
    List,
}

#[derive(Subcommand)]
enum ThemeAction {
    /// Set active theme
    Set {
        /// Theme name
        name: String,
    },
    /// List available themes
    List,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Validate configuration file
    Validate,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => cmd_init(force),
        Commands::Plugin { action } => match action {
            PluginAction::Add { name } => cmd_plugin_add(&name),
            PluginAction::Remove { name } => cmd_plugin_remove(&name),
            PluginAction::List => cmd_plugin_list(),
        },
        Commands::Theme { action } => match action {
            ThemeAction::Set { name } => cmd_theme_set(&name),
            ThemeAction::List => cmd_theme_list(),
        },
        Commands::Config { action } => match action {
            ConfigAction::Validate => cmd_config_validate(),
        },
    }
}

fn cmd_init(force: bool) -> Result<()> {
    let config_path = Config::default_path()?;

    if config_path.exists() && !force {
        eprintln!("Configuration already exists at {:?}", config_path);
        eprintln!("Use --force to overwrite");
        return Ok(());
    }

    let mut config = Config::default();
    // Ensure default theme is set
    if config.theme.is_none() {
        config.theme = Some("default".to_string());
    }
    config.save()?;

    println!("Initialized Ziron configuration at {:?}", config_path);
    if let Some(theme) = &config.theme {
        println!("Default theme set to: {}", theme);
    }
    Ok(())
}

fn cmd_plugin_add(name: &str) -> Result<()> {
    let mut config = Config::load().unwrap_or_default();

    if !config.modules.contains(&name.to_string()) {
        config.modules.push(name.to_string());
        config.save()?;
        println!("Added plugin: {}", name);
    } else {
        println!("Plugin already installed: {}", name);
    }

    Ok(())
}

fn cmd_plugin_remove(name: &str) -> Result<()> {
    let mut config = Config::load().unwrap_or_default();

    if let Some(pos) = config.modules.iter().position(|m| m == name) {
        config.modules.remove(pos);
        config.save()?;
        println!("Removed plugin: {}", name);
    } else {
        println!("Plugin not found: {}", name);
    }

    Ok(())
}

fn cmd_plugin_list() -> Result<()> {
    let config = Config::load().unwrap_or_default();

    if config.modules.is_empty() {
        println!("No plugins installed");
    } else {
        println!("Installed plugins:");
        for module in &config.modules {
            println!("  - {}", module);
        }
    }

    Ok(())
}

fn cmd_theme_set(name: &str) -> Result<()> {
    let mut config = Config::load().unwrap_or_default();
    config.theme = Some(name.to_string());
    config.save()?;
    println!("Set theme to: {}", name);
    Ok(())
}

fn cmd_theme_list() -> Result<()> {
    println!("Available themes:");
    println!("  - default");
    println!("  - minimal");
    Ok(())
}

fn cmd_config_validate() -> Result<()> {
    let config = Config::load()?;
    println!("Configuration is valid");
    println!("Shell: {}", config.shell.default);
    println!("Cache TTL: {}ms", config.performance.cache_ttl_ms);
    println!("Modules: {:?}", config.modules);
    if let Some(theme) = &config.theme {
        println!("Theme: {}", theme);
    }
    Ok(())
}

