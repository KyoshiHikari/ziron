//! Prompt rendering pipeline

use crate::error::Result;
use crate::module::{ModuleContext, ModuleData};
use crate::theme::Theme;

/// Prompt renderer
#[derive(Clone)]
pub struct PromptRenderer {
    theme: Theme,
}

impl PromptRenderer {
    /// Create a new prompt renderer with a theme
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Render a prompt from module data
    pub fn render(&self, _context: &ModuleContext, modules: &[ModuleData]) -> Result<String> {
        let mut output = String::new();

        // Set background color if specified
        if let Some(bg_color) = &self.theme.config.background {
            output.push_str(&self.hex_to_bg_ansi(bg_color));
        }

        let mut segments = Vec::new();

        // Render main prompt segments
        for segment_config in &self.theme.segments {
            // Find module data for this segment
            if let Some(module_data) = modules.iter().find(|m| m.module == segment_config.module) {
                let segment = self.render_segment(segment_config, module_data)?;
                segments.push(segment);
            } else {
                // If module not found, skip silently (for optional modules like git)
                // This allows themes to include optional modules
            }
        }

        // Handle multi-line prompts
        if self.theme.config.multiline.unwrap_or(false) {
            // For multi-line, join segments with newlines
            output.push_str(&segments.join("\n"));
        } else {
            // Single line
            output.push_str(&segments.join(""));
        }
        
        // Render right-side prompt if specified
        if let Some(right_segments) = &self.theme.config.right_segments {
            let right_prompt = self.render_right_prompt(right_segments, modules)?;
            if !right_prompt.is_empty() {
                // Calculate terminal width (default to 80 if not available)
                let terminal_width: usize = 80; // TODO: Get actual terminal width
                let left_prompt_len: usize = output.len(); // Approximate
                let padding = terminal_width.saturating_sub(left_prompt_len + right_prompt.len());
                output.push_str(&" ".repeat(padding));
                output.push_str(&right_prompt);
            }
        }
        
        // Reset background right after the last "#" so it includes the "#" but not the trailing space
        // We need to find the actual "#" character in the visible text (ignoring ANSI codes)
        if self.theme.config.background.is_some() {
            // Parse the string to find the last "#" while skipping ANSI escape sequences
            let mut last_hash_byte_pos: Option<usize> = None;
            let mut i = 0;
            let bytes = output.as_bytes();
            
            while i < bytes.len() {
                if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                    // Skip ANSI escape sequence: \x1b[...m
                    i += 2; // Skip \x1b[
                    while i < bytes.len() && bytes[i] != b'm' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1; // Skip 'm'
                    }
                } else {
                    // Check if this is the "#" character
                    if bytes[i] == b'#' {
                        last_hash_byte_pos = Some(i);
                    }
                    i += 1;
                }
            }
            
            if let Some(last_hash_byte_pos) = last_hash_byte_pos {
                // Find the byte position right after the "#"
                let pos_after_hash = last_hash_byte_pos + 1;
                
                // Check if there's a space after the "#"
                if pos_after_hash < bytes.len() && bytes[pos_after_hash] == b' ' {
                    // Insert reset code right after "#" but before the space
                    // Split at byte level to avoid UTF-8 issues
                    let before_hash = &output[..pos_after_hash];
                    let after_hash = &output[pos_after_hash..];
                    output = format!("{}{}{}", before_hash, "\x1b[0m", after_hash);
                } else {
                    // No space immediately after, reset at the end
                    output.push_str("\x1b[0m");
                }
            } else {
                // No "#" found, reset at the end
                output.push_str("\x1b[0m");
            }
        }
        
        Ok(output)
    }

    /// Render right-side prompt
    fn render_right_prompt(
        &self,
        right_segments: &[crate::theme::Segment],
        modules: &[ModuleData],
    ) -> Result<String> {
        let mut output = String::new();

        for segment_config in right_segments {
            // Find module data for this segment
            if let Some(module_data) = modules.iter().find(|m| m.module == segment_config.module) {
                let segment = self.render_segment(segment_config, module_data)?;
                output.push_str(&segment);
            }
        }

        Ok(output)
    }

    fn render_segment(
        &self,
        segment_config: &crate::theme::Segment,
        module_data: &ModuleData,
    ) -> Result<String> {
        let mut output = String::new();

        // Check rules for conditional display
        if !self.should_display_segment(segment_config, module_data)? {
            return Ok(String::new());
        }

        // Apply color if specified
        if let Some(color) = &segment_config.color {
            output.push_str(&self.color_to_ansi(color));
        }

        // Render module data
        if let Some(text) = module_data.data.get("text").and_then(|v| v.as_str()) {
            output.push_str(text);
        }

        // Reset foreground color (but keep background)
        if segment_config.color.is_some() {
            output.push_str("\x1b[39m"); // Reset foreground color only
            // Restore background if set
            if let Some(bg_color) = &self.theme.config.background {
                output.push_str(&self.hex_to_bg_ansi(bg_color));
            }
        } else {
            // Even if no color, restore background if set
            if let Some(bg_color) = &self.theme.config.background {
                output.push_str(&self.hex_to_bg_ansi(bg_color));
            }
        }

        // Add separator (background should already be active from above)
        if let Some(separator) = &segment_config.separator {
            // Ensure background is still active for the separator
            if let Some(bg_color) = &self.theme.config.background {
                output.push_str(&self.hex_to_bg_ansi(bg_color));
            }
            
            // If this is the last segment and separator contains "#", we need special handling
            // to reset background after "#" but before any trailing space
            if separator.contains('#') && separator.ends_with(' ') {
                // Find the position of "#" in the separator
                if let Some(hash_pos) = separator.rfind('#') {
                    // Add everything up to and including "#"
                    output.push_str(&separator[..hash_pos + 1]);
                    // Reset background right after "#"
                    output.push_str("\x1b[0m");
                    // Add the trailing space without background
                    if hash_pos + 1 < separator.len() {
                        output.push_str(&separator[hash_pos + 1..]);
                    }
                } else {
                    output.push_str(separator);
                }
            } else {
                output.push_str(separator);
            }
        }

        Ok(output)
    }

    /// Get color from palette or use directly
    fn get_color(&self, color: &str) -> String {
        // Check if color is in custom palette
        if let Some(palette) = &self.theme.config.color_palette {
            if let Some(palette_color) = palette.get(color) {
                return palette_color.clone();
            }
        }
        color.to_string()
    }

    fn color_to_ansi(&self, color: &str) -> String {
        let color = self.get_color(color);
        // Check if it's a hex color (true color)
        if color.starts_with('#') && color.len() == 7 {
            let hex = &color[1..];
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return format!("\x1b[38;2;{};{};{}m", r, g, b);
            }
        }
        
        // Check if it's RGB format: rgb(r,g,b)
        if color.starts_with("rgb(") && color.ends_with(')') {
            let rgb = &color[4..color.len()-1];
            let parts: Vec<&str> = rgb.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                ) {
                    return format!("\x1b[38;2;{};{};{}m", r, g, b);
                }
            }
        }
        
        // Standard color names
        let code = match color.to_lowercase().as_str() {
            "black" => "30",
            "red" => "31",
            "green" => "32",
            "yellow" => "33",
            "blue" => "34",
            "magenta" => "35",
            "cyan" => "36",
            "white" => "37",
            _ => "0",
        };
        format!("\x1b[{}m", code)
    }

    /// Convert hex color code to ANSI background color escape sequence
    fn hex_to_bg_ansi(&self, hex: &str) -> String {
        // Remove # if present
        let hex = hex.trim_start_matches('#');
        
        if hex.len() == 6 {
            // Parse RGB values
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return format!("\x1b[48;2;{};{};{}m", r, g, b);
            }
        }
        
        // Fallback: no background color
        String::new()
    }

    /// Check if a segment should be displayed based on rules
    fn should_display_segment(
        &self,
        segment_config: &crate::theme::Segment,
        module_data: &ModuleData,
    ) -> Result<bool> {
        if segment_config.rules.is_empty() {
            return Ok(true);
        }

        // Evaluate rules (simplified implementation)
        for rule in &segment_config.rules {
            // Simple rule evaluation - check if condition matches module data
            match rule.condition.as_str() {
                "if_exists" => {
                    if let Some(value) = rule.value.as_str() {
                        if !module_data.data.get(value).is_some() {
                            return Ok(false);
                        }
                    }
                }
                "if_not_empty" => {
                    if let Some(value) = rule.value.as_str() {
                        if let Some(data_value) = module_data.data.get(value) {
                            if let Some(str_val) = data_value.as_str() {
                                if str_val.is_empty() {
                                    return Ok(false);
                                }
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                }
                _ => {
                    // Unknown rule, default to showing
                }
            }
        }

        Ok(true)
    }
}

