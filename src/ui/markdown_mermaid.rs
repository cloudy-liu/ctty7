//! Mermaid diagram support for Markdown preview.
//!
//! This module processes Markdown text and converts Mermaid code blocks to SVG
//! diagrams using the merman library. The SVG diagrams are theme-aware and adapt
//! to the current UI theme colors.

use std::sync::Arc;
use regex::Regex;
use once_cell::sync::Lazy;

/// Configuration for Mermaid theme colors (derived from UI theme).
#[derive(Debug, Clone)]
pub struct MermaidConfig {
    pub primary_color: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    pub muted_color: String,
}

impl Default for MermaidConfig {
    fn default() -> Self {
        Self {
            primary_color: "#3b82f6".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#000000".to_string(),
            border_color: "#d1d5db".to_string(),
            muted_color: "#6b7280".to_string(),
        }
    }
}

/// Process Markdown text and convert Mermaid code blocks to inline SVG.
///
/// Finds all ```mermaid code blocks, renders them to SVG using merman,
/// and replaces them with the SVG output wrapped in appropriate HTML.
pub fn process_mermaid_blocks(markdown: &str, config: &MermaidConfig) -> String {
    static MERMAID_BLOCK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?s)```mermaid\s*\n(.*?)```").expect("valid regex")
    });

    let mut result = String::with_capacity(markdown.len());
    let mut last_end = 0;

    for cap in MERMAID_BLOCK.captures_iter(markdown) {
        let full_match = cap.get(0).unwrap();
        let diagram_code = cap.get(1).unwrap().as_str().trim();

        // Append text before this match
        result.push_str(&markdown[last_end..full_match.start()]);

        // Render the Mermaid diagram
        match render_mermaid_to_svg(diagram_code, config) {
            Ok(svg) => {
                // Wrap SVG in a container div for styling
                result.push_str("\n<div class=\"mermaid-diagram\">\n");
                result.push_str(&svg);
                result.push_str("\n</div>\n");
            }
            Err(e) => {
                // On error, keep original code block with error message
                result.push_str("```mermaid\n");
                result.push_str(diagram_code);
                result.push_str("\n```\n");
                result.push_str(&format!("\n> **Mermaid Error:** {}\n", e));
            }
        }

        last_end = full_match.end();
    }

    // Append remaining text
    result.push_str(&markdown[last_end..]);
    result
}

/// Render a single Mermaid diagram to SVG string.
fn render_mermaid_to_svg(diagram: &str, config: &MermaidConfig) -> Result<String, String> {
    // Build theme variables from config
    let theme_vars = build_theme_variables(config);

    // Create Merman configuration
    let mut merman_config = merman::Config::default();

    // Set theme to base (we'll override with our own colors)
    merman_config.theme = Some("base".to_string());

    // Apply our theme variables
    merman_config.theme_variables = Some(theme_vars.into_iter().collect());

    // Set default diagram config
    merman_config.flowchart = Some(merman::FlowchartConfig {
        use_max_width: Some(true),
        ..Default::default()
    });

    // Render to SVG
    merman::compile(diagram, merman_config)
        .map_err(|e| format!("{:?}", e))
}

/// Build Mermaid theme variables from config (derived from any theme's palette).
///
/// This maps the theme-agnostic config colors to Mermaid's specific theme variables.
/// The mapping is semantic: "accent" → primary, "border" → lines, etc.
fn build_theme_variables(config: &MermaidConfig) -> Vec<(String, String)> {
    vec![
        // Primary colors
        ("primaryColor".to_string(), config.primary_color.clone()),
        ("primaryTextColor".to_string(), config.text_color.clone()),
        ("primaryBorderColor".to_string(), config.border_color.clone()),

        // Background
        ("background".to_string(), config.background_color.clone()),
        ("mainBkg".to_string(), config.background_color.clone()),
        ("secondBkg".to_string(), lighten_color(&config.background_color, 0.05)),
        ("tertiaryBkg".to_string(), lighten_color(&config.background_color, 0.1)),

        // Text colors
        ("textColor".to_string(), config.text_color.clone()),
        ("secondaryTextColor".to_string(), config.muted_color.clone()),

        // Border and lines
        ("border1".to_string(), config.border_color.clone()),
        ("border2".to_string(), config.border_color.clone()),
        ("lineColor".to_string(), config.border_color.clone()),

        // Node-specific colors (flowcharts, etc.)
        ("nodeBkg".to_string(), config.background_color.clone()),
        ("nodeBorder".to_string(), config.border_color.clone()),
        ("clusterBkg".to_string(), lighten_color(&config.background_color, 0.03)),
        ("clusterBorder".to_string(), config.border_color.clone()),

        // Sequence diagram
        ("actorBkg".to_string(), config.background_color.clone()),
        ("actorBorder".to_string(), config.border_color.clone()),
        ("actorTextColor".to_string(), config.text_color.clone()),
        ("actorLineColor".to_string(), config.border_color.clone()),
        ("signalColor".to_string(), config.text_color.clone()),
        ("signalTextColor".to_string(), config.text_color.clone()),

        // State diagram
        ("labelColor".to_string(), config.text_color.clone()),
        ("labelBackgroundColor".to_string(), config.background_color.clone()),

        // Class diagram
        ("classText".to_string(), config.text_color.clone()),

        // Gantt chart
        ("sectionBkgColor".to_string(), lighten_color(&config.background_color, 0.05)),
        ("altSectionBkgColor".to_string(), config.background_color.clone()),
        ("sectionBkgColor2".to_string(), lighten_color(&config.background_color, 0.08)),
        ("taskBorderColor".to_string(), config.border_color.clone()),
        ("taskBkgColor".to_string(), config.primary_color.clone()),
        ("taskTextColor".to_string(), config.text_color.clone()),
        ("activeTaskBorderColor".to_string(), config.primary_color.clone()),
        ("activeTaskBkgColor".to_string(), config.primary_color.clone()),
        ("gridColor".to_string(), config.border_color.clone()),
        ("doneTaskBkgColor".to_string(), lighten_color(&config.border_color, 0.2)),
        ("doneTaskBorderColor".to_string(), config.border_color.clone()),
        ("critBkgColor".to_string(), darken_color(&config.primary_color, 0.2)),
        ("critBorderColor".to_string(), darken_color(&config.primary_color, 0.3)),
        ("todayLineColor".to_string(), config.primary_color.clone()),
    ]
}

/// Simple color lightening (increases RGB values proportionally).
fn lighten_color(hex: &str, amount: f32) -> String {
    adjust_color(hex, amount, true)
}

/// Simple color darkening (decreases RGB values proportionally).
fn darken_color(hex: &str, amount: f32) -> String {
    adjust_color(hex, -amount, false)
}

/// Adjust color brightness by given amount.
fn adjust_color(hex: &str, amount: f32, lighten: bool) -> String {
    // Remove # prefix if present
    let hex = hex.trim_start_matches('#');

    // Parse RGB
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    // Adjust
    let adjust = |v: u8| -> u8 {
        if lighten {
            let diff = 255 - v;
            v + (diff as f32 * amount) as u8
        } else {
            v - (v as f32 * amount.abs()) as u8
        }
    };

    let r = adjust(r);
    let g = adjust(g);
    let b = adjust(b);

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Extract Mermaid config from current theme.
pub fn mermaid_config_from_theme(cx: &gpui::App) -> MermaidConfig {
    let theme = cx.theme();

    MermaidConfig {
        primary_color: format_color(theme.accent),
        background_color: format_color(theme.surface.background),
        text_color: format_color(theme.foreground),
        border_color: format_color(theme.surface.border),
        muted_color: format_color(theme.foreground.opacity(0.6)),
    }
}

/// Convert GPUI color to hex string.
fn format_color(color: gpui::Hsla) -> String {
    let rgb = color.to_rgb();
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb.r * 255.0) as u8,
        (rgb.g * 255.0) as u8,
        (rgb.b * 255.0) as u8
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighten_color() {
        assert_eq!(lighten_color("#000000", 0.5), "#7f7f7f");
        assert_eq!(lighten_color("#ffffff", 0.5), "#ffffff");
    }

    #[test]
    fn test_darken_color() {
        assert_eq!(darken_color("#ffffff", 0.5), "#7f7f7f");
        assert_eq!(darken_color("#000000", 0.5), "#000000");
    }

    #[test]
    fn test_process_no_mermaid() {
        let input = "# Hello\n\nSome text\n\n```rust\nfn main() {}\n```";
        let config = MermaidConfig::default();
        assert_eq!(process_mermaid_blocks(input, &config), input);
    }

    #[test]
    fn test_process_with_mermaid() {
        let input = "# Test\n\n```mermaid\ngraph TD\n  A-->B\n```\n\nMore text";
        let config = MermaidConfig::default();
        let result = process_mermaid_blocks(input, &config);

        // Should contain SVG or error message
        assert!(result.contains("<div class=\"mermaid-diagram\">") || result.contains("Mermaid Error"));
    }
}
