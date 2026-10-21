# Native document tab evidence

The opt-in `--document-visual` fixture opens the full production `Tty7App` in a separate native window. It uses normal document-opening actions, awaits each file load and waits for Markdown parsing or the native HTML page before publishing readiness. It does not replace the app's renderer or document controls.

Build with `cargo build --bin tty7-app --features markdown-visual-tests`. Run each capture with its own scratch `--config-dir`, never the user's normal configuration directory. Fixture windows use the title `ctty7 document acceptance` so local computer use can identify them without touching other app windows.

Example options, saved as an absolute JSON file outside the checkout:

```json
{
  "root": "D:/gh-prj/ctty7-file-tabs/tests/fixtures/document-tabs",
  "files": [
    "docs/maintenance/migration.md",
    "docs/maintenance/release.md",
    "src/settings.rs",
    "overview.html"
  ],
  "active": 0,
  "preview": true,
  "width": 1200,
  "height": 900,
  "dark": true,
  "dock": false,
  "output": "D:/gh-prj/document-preview-ready.json",
  "image": "D:/gh-prj/document-preview.png",
  "hold_seconds": 120
}
```

```powershell
Start-Process -WindowStyle Normal -FilePath D:/gh-prj/ctty7-tabs-after.exe -ArgumentList '--config-dir', 'D:/gh-prj/document-preview-config', '--document-visual', 'D:/gh-prj/document-preview-options.json' -RedirectStandardError D:/gh-prj/document-preview.log
```

Remove any previous readiness output before launching. Wait for `NATIVE_DOCUMENT_READY_FOR_CAPTURE` in stderr and `"ready": true` in the output JSON, then capture the fixture's native window. The JSON records the actual viewport, OS scaling, opened documents, selected file and mode. Its `capture` field states whether the requested GPU image was saved or a native screenshot is needed. The fixture holds for the requested duration, capped at 600 seconds, then exits.

For Markdown source use `active: 0, preview: false`; for ordinary source use `active: 2, preview: false`; for HTML use `active: 3, preview: true`. Omitting `preview` retains the production first-open default. Use `dock: true` to inspect the docked document layout. Keep the files, active document, mode, theme, viewport, OS scaling and screenshot method identical in each Before/After pair.

## Baseline reproduction

Create an isolated worktree at `e06271fb` and apply the native evidence fixture and window-registration fix with `git cherry-pick 89176e54 8de7b3f5`. These commits add the fixture, its CLI dispatch, sample documents, these instructions and registration that keeps the fixture window open when the tray is disabled. They do not change production document UI. Build there and copy the executable outside `target` before another build can overwrite it. Run baseline and final binaries sequentially against the same fixture root and separate scratch configuration directories. The fixture is intentionally visible for native inspection and capture, so launch it with `-WindowStyle Normal`.

Keep the captured PNGs in the repository and embed them directly in the PR's Evidence section with repository image URLs. A local file link alone does not give reviewers a visible comparison.

## Capture limits

GPUI's current macOS backend supports native `render_to_image`. The current Windows native backend reports that this operation is unsupported, so use Local Computer Use to inspect and capture the fixture window. An HTML preview is a native child WebView and is absent from GPUI GPU output on any platform; always use a native window screenshot for HTML. Linux retains the production HTML source fallback. The fixture's ready marker proves content load, while visual inspection establishes that controls and content are visible and aligned.
