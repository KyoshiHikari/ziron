//! Ziron Shell - A modern, extensible shell interpreter

use ziron_core::config::Config;
use ziron_core::error::Result;
use ziron_core::prompt::PromptRenderer;
use ziron_core::theme::Theme;

mod command;
mod completion;
mod executor;
mod jobs;
mod parser;
mod shell;

use shell::ZironShell;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = Config::load().unwrap_or_default();

    // Load theme
    let theme = if let Some(theme_name) = &config.theme {
        let theme_path = std::path::PathBuf::from("themes").join(theme_name).join("theme.toml");
        Theme::load_from(&theme_path)?
    } else {
        Theme::load_from(&Theme::default_path()?)?
    };

    let renderer = PromptRenderer::new(theme);

    // Create shell instance
    let mut shell = ZironShell::new(config, renderer)?;

    // Run shell
    shell.run()?;

    Ok(())
}

