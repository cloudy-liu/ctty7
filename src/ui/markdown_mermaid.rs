//! Mermaid diagram rendering for the Markdown preview.
//!
//! Diagrams are rendered to SVG by the pure-Rust `merman` renderer. The
//! colors come from the active Markdown theme palette, so any theme (built-in
//! Paperglow or a user YAML theme) drives diagrams in both light and dark mode
//! without Mermaid-specific theme fields.

use merman::render::{HeadlessRenderer, HostThemeAppearance, HostThemeProfile, HostThemeRoles};

use crate::core::markdown_theme::{Color, Theme};

/// Render Mermaid `source` to an SVG string styled with `theme`'s palette.
pub fn render_mermaid(source: &str, theme: &Theme, dark: bool) -> Result<String, String> {
    HeadlessRenderer::new()
        .render_svg_with_host_theme_sync(source, &build_theme_profile(theme, dark))
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "not a recognized Mermaid diagram".to_string())
}

/// Map the Markdown theme palette onto merman's semantic theme roles.
fn build_theme_profile(theme: &Theme, dark: bool) -> HostThemeProfile {
    let p = theme.palette(dark);
    let hex = |c: &Color| Some(color_to_hex(c));

    HostThemeProfile::builder()
        .appearance(if dark {
            HostThemeAppearance::Dark
        } else {
            HostThemeAppearance::Light
        })
        .roles(HostThemeRoles {
            canvas: hex(&p.background),
            surface: hex(&p.paper),
            surface_alt: None,
            surface_muted: None,
            text: hex(&p.foreground),
            subtle_text: hex(&p.quote_foreground),
            border: hex(&p.border),
            line: hex(&p.border),
            edge_label_background: hex(&p.paper),
            cluster_background: hex(&p.paper),
            cluster_border: hex(&p.border),
            note_background: hex(&p.quote_background),
            note_border: hex(&p.quote_border),
            note_text: hex(&p.quote_foreground),
            actor_background: hex(&p.code_background),
            actor_border: hex(&p.border),
            actor_text: hex(&p.foreground),
            activation_background: hex(&p.accent),
            activation_border: hex(&p.accent),
            // Semantic colors stay fixed across themes.
            error: Some("#EF4444".to_string()),
            warning: Some("#F59E0B".to_string()),
            success: Some("#10B981".to_string()),
        })
        .build()
}

fn color_to_hex(color: &Color) -> String {
    let [r, g, b, a] = color.0.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
    if a == u8::MAX {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_to_hex_formats_opaque_and_translucent() {
        assert_eq!(color_to_hex(&Color([1.0, 0.0, 0.5, 1.0])), "#FF0080");
        assert_eq!(color_to_hex(&Color([0.0, 0.0, 0.0, 0.5])), "#00000080");
    }
}
