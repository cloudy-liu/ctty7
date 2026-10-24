use resvg::tiny_skia;
use resvg::usvg;

pub(super) struct RgbaImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[cfg(target_os = "macos")]
const GLYPH_SVG: &[u8] = include_bytes!("../../../assets/tray.svg");
#[cfg(not(target_os = "macos"))]
const GLYPH_SVG: &[u8] = include_bytes!("../../../assets/logo.svg");

#[cfg(target_os = "macos")]
const SIZE: u32 = 36;
#[cfg(not(target_os = "macos"))]
const SIZE: u32 = 32;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const AMBER: (u8, u8, u8) = (0xF5, 0x9E, 0x0B);

#[cfg(target_os = "macos")]
pub(super) fn render() -> Option<RgbaImage> {
    let tree = usvg::Tree::from_data(GLYPH_SVG, &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(SIZE, SIZE)?;
    resvg::render(&tree, fit_center(&tree, SIZE), &mut pixmap.as_mut());
    Some(to_rgba(&pixmap))
}

#[cfg(not(target_os = "macos"))]
pub(super) fn render(attention: bool) -> Option<RgbaImage> {
    render_sized(attention, SIZE)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn render_sized(attention: bool, size: u32) -> Option<RgbaImage> {
    let mut pixmap = render_glyph(size)?;

    if attention {
        badge(&mut pixmap);
    }

    Some(to_rgba(&pixmap))
}

#[cfg(not(target_os = "macos"))]
fn render_glyph(size: u32) -> Option<tiny_skia::Pixmap> {
    let tree = usvg::Tree::from_data(GLYPH_SVG, &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(size, size)?;
    resvg::render(&tree, fit_center(&tree, size), &mut pixmap.as_mut());
    Some(pixmap)
}

#[cfg(target_os = "windows")]
pub(super) fn render_taskbar(size: u32) -> Option<RgbaImage> {
    let mut pixmap = render_glyph(size)?;
    let s = size as f32;
    paint_attention(&mut pixmap, s * 0.68, s * 0.32, s * 0.30, 0.22);
    Some(to_rgba(&pixmap))
}

pub(super) fn agent_avatar(
    agent: crate::core::cli_agent::CLIAgent,
    status: crate::core::cli_agent::AgentStatus,
) -> Option<tiny_skia::Pixmap> {
    use gpui::AssetSource as _;

    const SIZE: u32 = 32;
    let s = SIZE as f32;
    let mut pixmap = tiny_skia::Pixmap::new(SIZE, SIZE)?;

    let accent = agent.accent_rgb();
    let mut paint = tiny_skia::Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color_rgba8(
        (accent >> 16) as u8,
        (accent >> 8) as u8,
        accent as u8,
        0xFF,
    );
    let mut pb = tiny_skia::PathBuilder::new();
    pb.push_circle(s / 2.0, s / 2.0, s / 2.0);
    let disc = pb.finish()?;
    pixmap.fill_path(
        &disc,
        &paint,
        tiny_skia::FillRule::Winding,
        tiny_skia::Transform::identity(),
        None,
    );

    let svg = crate::ui::assets::Assets
        .load(agent.icon_path())
        .ok()
        .flatten()?;
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default()).ok()?;
    let glyph_size = (s * 0.60).round() as u32;
    let mut glyph = tiny_skia::Pixmap::new(glyph_size, glyph_size)?;
    resvg::render(&tree, fit_center(&tree, glyph_size), &mut glyph.as_mut());
    recolor(&mut glyph, (0xFF, 0xFF, 0xFF));
    let offset = ((SIZE - glyph_size) / 2) as i32;
    pixmap.draw_pixmap(
        offset,
        offset,
        glyph.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );

    // The same symbol the tab avatars wear. A tray menu's background belongs to
    // the OS, so the symbol brings its own surface, the one herdr's dark
    // colours are drawn for.
    use crate::ui::status_indicator::StatusIndicator;
    let indicator = StatusIndicator::of_status(status);
    let (cx, cy, r) = (s * 0.78, s * 0.78, s * 0.22);
    let circle = |radius: f32| {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_circle(cx, cy, radius);
        pb.finish()
    };
    if let Some(ring) = circle(r * 1.25) {
        paint.blend_mode = tiny_skia::BlendMode::Clear;
        pixmap.fill_path(
            &ring,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }
    if let Some(disc) = circle(r) {
        paint.blend_mode = tiny_skia::BlendMode::SourceOver;
        let [_, red, green, blue] = StatusIndicator::DARK_SURFACE.to_be_bytes();
        paint.set_color_rgba8(red, green, blue, 0xFF);
        pixmap.fill_path(
            &disc,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }
    let svg = crate::ui::assets::Assets
        .load(indicator.icon_path())
        .ok()
        .flatten()?;
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default()).ok()?;
    let mark_size = (r * 2.0).round() as u32;
    let mut mark = tiny_skia::Pixmap::new(mark_size, mark_size)?;
    resvg::render(&tree, fit_center(&tree, mark_size), &mut mark.as_mut());
    let rgb = indicator.rgb(true);
    recolor(&mut mark, ((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8));
    pixmap.draw_pixmap(
        (cx - mark_size as f32 / 2.0).round() as i32,
        (cy - mark_size as f32 / 2.0).round() as i32,
        mark.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );

    Some(pixmap)
}

fn fit_center(tree: &usvg::Tree, size: u32) -> tiny_skia::Transform {
    let (w, h) = (tree.size().width(), tree.size().height());
    let scale = size as f32 / w.max(h);
    tiny_skia::Transform::from_scale(scale, scale).post_translate(
        (size as f32 - w * scale) / 2.0,
        (size as f32 - h * scale) / 2.0,
    )
}

pub(super) fn to_rgba(pixmap: &tiny_skia::Pixmap) -> RgbaImage {
    let mut data = Vec::with_capacity(pixmap.data().len());
    for p in pixmap.pixels() {
        let c = p.demultiply();
        data.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    RgbaImage {
        data,
        width: pixmap.width(),
        height: pixmap.height(),
    }
}

fn recolor(pixmap: &mut tiny_skia::Pixmap, rgb: (u8, u8, u8)) {
    for p in pixmap.pixels_mut() {
        let a = p.alpha();
        let mul = |c: u8| ((c as u16 * a as u16) / 255) as u8;
        if let Some(np) =
            tiny_skia::PremultipliedColorU8::from_rgba(mul(rgb.0), mul(rgb.1), mul(rgb.2), a)
        {
            *p = np;
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn badge(pixmap: &mut tiny_skia::Pixmap) {
    let s = pixmap.width() as f32;
    let (cx, cy) = (s * 0.78, s * 0.22);
    let r = s * 0.20;
    let circle = |radius: f32| {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_circle(cx, cy, radius);
        pb.finish()
    };
    let mut paint = tiny_skia::Paint {
        anti_alias: true,
        ..Default::default()
    };

    if let Some(ring) = circle(r * 1.35) {
        paint.blend_mode = tiny_skia::BlendMode::Clear;
        pixmap.fill_path(
            &ring,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }
    if let Some(dot) = circle(r) {
        paint.blend_mode = tiny_skia::BlendMode::SourceOver;
        paint.set_color_rgba8(AMBER.0, AMBER.1, AMBER.2, 0xFF);
        pixmap.fill_path(
            &dot,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}

#[cfg(target_os = "windows")]
fn paint_attention(pixmap: &mut tiny_skia::Pixmap, x: f32, y: f32, radius: f32, dot_scale: f32) {
    for (radius, color) in [(radius, 0x34), (radius * dot_scale, 0xFF)] {
        let mut path = tiny_skia::PathBuilder::new();
        path.push_circle(x, y, radius);
        let mut paint = tiny_skia::Paint {
            anti_alias: true,
            ..Default::default()
        };
        paint.set_color_rgba8(color, color, color, 0xFF);
        if let Some(path) = path.finish() {
            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn badge(pixmap: &mut tiny_skia::Pixmap) {
    let s = pixmap.width() as f32;
    paint_attention(pixmap, s * 0.73, s * 0.27, s * 0.24, 0.26);
}

#[cfg(target_os = "linux")]
pub(super) fn render_argb(attention: bool) -> Option<(Vec<u8>, u32)> {
    let img = render(attention)?;
    let mut argb = Vec::with_capacity(img.data.len());
    for px in img.data.chunks_exact(4) {
        argb.extend_from_slice(&[px[3], px[0], px[1], px[2]]);
    }
    Some((argb, img.width))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cli_agent::{AgentStatus, CLIAgent};

    #[cfg(target_os = "windows")]
    #[test]
    fn taskbar_attention_restores_overlay_size_and_the_tray_badge_is_enlarged() {
        for size in [24, 32, 48, 64, 256] {
            let taskbar = render_taskbar(size).unwrap();
            let tray = render_sized(true, size).unwrap();
            let normal = render_sized(false, size).unwrap();
            let pixel = |image: &RgbaImage, x: f32, y: f32| {
                let px = ((size as f32 * x).round() as u32).min(size - 1);
                let py = ((size as f32 * y).round() as u32).min(size - 1);
                let offset = ((py * size + px) * 4) as usize;
                <[u8; 4]>::try_from(&image.data[offset..offset + 4]).unwrap()
            };
            // This lies inside the taskbar disc, outside the tray disc.
            assert_eq!(
                pixel(&taskbar, 0.40, 0.34),
                [52, 52, 52, 255],
                "size {size}"
            );
            assert_eq!(pixel(&tray, 0.40, 0.34), pixel(&normal, 0.40, 0.34));
            assert_eq!(pixel(&taskbar, 0.66, 0.34), [255, 255, 255, 255]);
            assert_eq!(pixel(&tray, 0.73, 0.27), [255, 255, 255, 255]);
            assert_eq!(pixel(&tray, 0.52, 0.26), [52, 52, 52, 255]);
            assert_ne!(pixel(&tray, 0.52, 0.26), pixel(&normal, 0.52, 0.26));
            assert_eq!(pixel(&taskbar, 0., 0.), [0, 0, 0, 0]);
            // The circle retains a small margin instead of being clipped at the edge.
            assert!(pixel(&taskbar, 0.99, 0.34)[3] < 255, "size {size}");
            for y in (size as f32 * 0.67).ceil() as u32..size {
                let offset = (y * size * 4) as usize;
                assert_eq!(
                    &taskbar.data[offset..offset + (size * 4) as usize],
                    &normal.data[offset..offset + (size * 4) as usize],
                    "lower artwork changed at size {size}"
                );
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_attention_artwork_keeps_the_mark_upper_right_at_every_size() {
        for size in [16, 32, 48, 64, 256] {
            let normal = render_sized(false, size).unwrap();
            let image = render_sized(true, size).unwrap();
            let pixel = |image: &RgbaImage, x: u32, y: u32| {
                let offset = ((y * size + x) * 4) as usize;
                <[u8; 4]>::try_from(&image.data[offset..offset + 4]).unwrap()
            };
            let x = (size as f32 * 0.73) as u32;
            let y = (size as f32 * 0.27) as u32;
            assert_eq!(pixel(&image, x, y), [255, 255, 255, 255], "size {size}");
            assert_eq!(
                pixel(&image, x, (size as f32 * 0.46).round() as u32),
                [52, 52, 52, 255]
            );
            assert_eq!(pixel(&image, 0, 0), [0, 0, 0, 0]);
            for y in (size as f32 * 0.55).ceil() as u32..size {
                for x in 0..size {
                    assert_eq!(
                        pixel(&image, x, y),
                        pixel(&normal, x, y),
                        "lower half changed at size {size}"
                    );
                }
            }
        }
    }

    #[test]
    fn recolor_keeps_alpha_and_flattens_color() {
        let mut pm = tiny_skia::Pixmap::new(2, 2).unwrap();
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(10, 200, 30, 128);
        pm.fill_rect(
            tiny_skia::Rect::from_xywh(0.0, 0.0, 2.0, 2.0).unwrap(),
            &paint,
            tiny_skia::Transform::identity(),
            None,
        );
        let alphas: Vec<u8> = pm.pixels().iter().map(|p| p.alpha()).collect();
        recolor(&mut pm, (0xFF, 0xFF, 0xFF));
        for (p, a) in pm.pixels().iter().zip(alphas) {
            assert_eq!(p.alpha(), a);
            assert!(p.red().abs_diff(a) <= 1, "red {} vs alpha {a}", p.red());
            assert_eq!(p.red(), p.green());
            assert_eq!(p.green(), p.blue());
        }
    }

    #[test]
    fn to_rgba_demultiplies() {
        let mut pm = tiny_skia::Pixmap::new(1, 1).unwrap();
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(255, 0, 0, 128);
        pm.fill_rect(
            tiny_skia::Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap(),
            &paint,
            tiny_skia::Transform::identity(),
            None,
        );
        let img = to_rgba(&pm);
        assert_eq!((img.width, img.height), (1, 1));
        let px = &img.data[0..4];
        assert_eq!(px[3], 128);
        assert!(px[0] >= 253, "red demultiplied back to ~255, got {}", px[0]);
        assert_eq!((px[1], px[2]), (0, 0));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn render_produces_template_glyph() {
        let img = render().unwrap();
        assert_eq!((img.width, img.height), (SIZE, SIZE));
        assert_eq!(img.data.len(), (SIZE * SIZE * 4) as usize);
        let covered = img.data.chunks_exact(4).filter(|p| p[3] > 0).count();
        assert!(covered > 0, "icon rendered fully transparent");
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn render_produces_both_states() {
        let normal = render(false).unwrap();
        let attention = render(true).unwrap();
        for img in [&normal, &attention] {
            assert_eq!((img.width, img.height), (SIZE, SIZE));
            assert_eq!(img.data.len(), (SIZE * SIZE * 4) as usize);
            let covered = img.data.chunks_exact(4).filter(|p| p[3] > 0).count();
            assert!(covered > 0, "icon rendered fully transparent");
        }
        assert_ne!(normal.data, attention.data);
    }

    #[test]
    fn agent_avatar_renders_brand_and_fallback() {
        for agent in CLIAgent::ALL {
            let idle = agent_avatar(agent, AgentStatus::Idle).unwrap();
            let waiting = agent_avatar(agent, AgentStatus::Waiting).unwrap();
            assert_eq!((idle.width(), idle.height()), (32, 32));
            assert_eq!(idle.pixel(0, 0).unwrap().alpha(), 0);
            assert!(idle.pixel(16, 16).unwrap().alpha() > 0);
            let shades: std::collections::HashSet<_> = idle
                .pixels()
                .iter()
                .filter(|p| p.alpha() == 0xFF)
                .map(|p| (p.red(), p.green(), p.blue()))
                .collect();
            assert!(
                shades.len() > 1,
                "{} rendered as a bare disc — its glyph drew nothing",
                agent.display_name()
            );
            assert_ne!(idle.data(), waiting.data());
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn render_argb_reorders_bytes() {
        let rgba = render(false).unwrap();
        let (argb, size) = render_argb(false).unwrap();
        assert_eq!(size, rgba.width);
        assert_eq!(argb.len(), rgba.data.len());
        for (a4, r4) in argb.chunks_exact(4).zip(rgba.data.chunks_exact(4)) {
            assert_eq!(a4, [r4[3], r4[0], r4[1], r4[2]]);
        }
    }
}
