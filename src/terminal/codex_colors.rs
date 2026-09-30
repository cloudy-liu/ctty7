//! Paint-time compatibility for Codex's cached terminal palette.
//!
//! Codex 0.159.2 caches OSC 10/11 at startup. Its composer/history blends and
//! built-in diff fills then remain RGB literals even after tty7 changes theme.
//! Recognize those exact fills only in Codex panes, including nested agents.
//! Never rewrite the terminal grid: scrollback and repeated theme swaps must
//! always resolve from the original output, without accumulating color drift.
//!
//! Source: openai/codex, rust-v0.159.2, codex-rs/tui/src/{style,diff_render}.rs.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ui::presets::{contrast, mix};

#[derive(Clone)]
pub(crate) struct CodexPalette(Arc<HashMap<u32, u32>>);

impl gpui::Global for CodexPalette {}

impl CodexPalette {
    pub(crate) fn new(backgrounds: impl IntoIterator<Item = u32>, current: u32) -> Self {
        let mut fills = HashMap::new();
        // Known theme backgrounds also recognize restored panes whose startup
        // OSC exchange predates this GUI. Black/white cover terminal fallbacks.
        // Insert the current theme last to preserve its native fills if two
        // source themes happen to produce the same rounded RGB value.
        for bg in [0x000000, 0xffffff]
            .into_iter()
            .chain(backgrounds)
            .chain([current])
        {
            fills.insert(prompt_fill(bg, false), prompt_fill(current, false));
            fills.insert(prompt_fill(bg, true), prompt_fill(current, true));
        }
        let light = is_light(current);
        for (dark, pale, gutter) in [
            (0x213a2b, 0xdafbe1, 0xaceebb),
            (0x4a221d, 0xffebe9, 0xffcecb),
        ] {
            fills.insert(dark, if light { pale } else { dark });
            fills.insert(pale, if light { pale } else { dark });
            fills.insert(gutter, if light { gutter } else { dark });
        }
        Self(Arc::new(fills))
    }

    pub(crate) fn background(&self, source: u32) -> Option<u32> {
        self.0.get(&source).copied()
    }
}

fn is_light(rgb: u32) -> bool {
    let r = ((rgb >> 16) & 255) as f32;
    let g = ((rgb >> 8) & 255) as f32;
    let b = (rgb & 255) as f32;
    0.299 * r + 0.587 * g + 0.114 * b > 128.0
}

fn prompt_fill(bg: u32, history: bool) -> u32 {
    let (top, alpha) = match (is_light(bg), history) {
        (true, false) => (0.0, 0.04),
        (true, true) => (0.0, 0.02),
        (false, false) => (255.0, 0.12),
        (false, true) => (255.0, 0.16),
    };
    // Match Codex's truncation, rather than presets::mix's rounding.
    [16, 8, 0].into_iter().fold(0, |out, shift| {
        let channel = ((bg >> shift) & 255) as f32;
        out | (((top * alpha + channel * (1.0 - alpha)) as u32) << shift)
    })
}

/// Preserve readable syntax ink; otherwise move it toward black or white.
/// Measure the rendered ink, including SGR dim, against the resolved fill.
pub(crate) fn readable_ink(fg: u32, bg: u32, opacity: f32) -> u32 {
    let visible_contrast = |ink| contrast(mix(bg, ink, opacity), bg);
    if visible_contrast(fg) >= 4.5 {
        return fg;
    }
    let toward = if visible_contrast(0xffffff) >= visible_contrast(0x000000) {
        0xffffff
    } else {
        0x000000
    };
    if visible_contrast(toward) < 4.5 {
        return toward;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..16 {
        let mid = (lo + hi) / 2.0;
        if visible_contrast(mix(fg, toward, mid)) >= 4.5 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    mix(fg, toward, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_theme_and_same_mode_switches_resolve_from_original_rgb() {
        // Custom light parchment -> custom dark -> white -> custom dark.
        // Include both startup palettes, as a restored pane's scrollback can.
        for (target, composer, history) in [
            (0x202830, 0x3a4148, 0x434a51),
            (0xffffff, 0xf4f4f4, 0xf9f9f9),
            (0x202830, 0x3a4148, 0x434a51),
            (0x282c34, 0x41454c, 0x4a4d54),
        ] {
            let palette = CodexPalette::new([0xf0e0d0, 0x202830], target);
            for bg in [0xe6d7c7, 0x3a4148] {
                assert_eq!(palette.background(bg), Some(composer));
            }
            for bg in [0xebdbcb, 0x434a51] {
                assert_eq!(palette.background(bg), Some(history));
            }
            assert_eq!(palette.background(0x123456), None);
        }
    }
}
