/// Render mermaid diagram source to SVG string
pub fn render_mermaid(source: &str) -> Option<String> {
    use merman::render::HeadlessRenderer;

    let renderer = HeadlessRenderer::new();
    renderer.render_svg_sync(source).ok().flatten()
}
