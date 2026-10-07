# v0.2.0 local review record

Prepared on 2026-10-07 and refreshed on 2026-10-08. The user has authorized
committing, building, and publishing v0.2.0 after verification. The release must
remain a draft until its complete asset set and checksums pass.

## Scope and tracker state

- Repository: `cloudy-liu/ctty7`; release branch: `codex/release-v0.2.0`.
- Refreshed starting commit: `8fcc511a49160ee5278695c11723565990790480`, matching fetched fork main on 2026-10-08. The original preparation used `b4f7b086`.
- Previous published release: v0.1.0, published 2026-09-29 and still Latest.
- Before release preparation there were no open PRs. Release preparation is tracked in [PR #115](https://github.com/cloudy-liu/ctty7/pull/115); it closes #100, the compact sidebar status spec, when merged.
- PR #103 already implements #100, and child issue #101 is closed. The remaining native checks and stale status documentation are addressed in the [acceptance record](sidebar-status-tags.md).
- The full history since v0.1.0 was reviewed, including the baseline merge and patch-equivalent terminal-color history reconciliation.
- PRs #112, #113, and #114 add diff width hardening, stable Cursor hook/session identity, and a coherent GitHub diff palette. Their changes and remaining acceptance gaps are included in the release notes.

## Local changes

- Workspace version and the four workspace packages in Cargo.lock now use 0.2.0. Third-party dependency versions are unchanged.
- [English README](../../README.md) and [Chinese README](../../README.zh-CN.md) are rewritten around current installation, interface controls, agent states, document editing, and fork-specific workflows.
- One combined light/dark agent-terminal image replaces the inherited README hero, alongside a separate source-editor screenshot. [Capture details](../../assets/screenshots/README.md) identify the real CLIs, sample sidebar hook events, and source build.
- Both READMEs retain the original pink typing-title animation. The logo and title share a centered paragraph with line breaks and image-margin overrides for preview compatibility.
- [English release notes](../releases/v0.2.0.md) follow the previous GitHub Release structure and the current AGENTS.md rules.
- The CLI endpoint-resolution test reuses its existing platform shell helper instead of requiring `sh` on Windows. Its expected exit code and endpoint assertions are unchanged.
- Status documentation now covers uppercase tags, the status-text option, and Codex's ready prompt.
- The cached Markdown view now fills the viewport, observes layout without adding scrollable blank space, and clamps the scroll offset when a document shrinks.
- The built-in GitHub dark reading theme follows the application's neutral foreground while preserving semantic colors and custom-theme palettes.

## Verification

The format, workspace check, complete workspace test suite, desktop updater
tests, and version-metadata checks were rerun successfully on 2026-10-08 with
all changes through `8fcc511a` and this release preparation. Four workspace
packages report 0.2.0; third-party dependency versions are unchanged. Logs are
retained in the ignored `.tty7-dev-release/release-v0.2.0-*.log` files.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo check --locked --workspace` | Passed. |
| `cargo test --locked --workspace` | Passed, with eight existing ignored tests. |
| `cargo test --locked --features updater --bin tty7-updater` | All 17 tests passed. |
| `git diff --check` | Passed. |
| Markdown scroll and dark foreground regressions | Passed as part of the full workspace suite, including dock/fill bounds, the final block, short-document clamping, live theme changes, Rust neutral tokens, and custom-theme preservation. |
| Native app | The linked executable reports file version 0.2.0; sidebar, Markdown/Mermaid, source/preview switching, and light/dark appearance were exercised. |
| README assets | The combined light/dark hero preserves both native screenshot regions pixel for pixel; Claude Code's orange mascot was captured after removing inherited NO_COLOR from the isolated screenshot processes. The source-editor WebP also reproduces its original capture. |
| README preview | The current Chinese preview loads the combined agent hero, logo, and source-editor screenshot without page overflow at 372 pixels. Both READMEs' local image and document paths were checked. |
| README header alignment and spacing | Typora's Chinese preview displays the logo and animated title on the same vertical centerline with reduced spacing. The browser preview measures a 29.7 CSS-pixel gap, matching the current GitHub README; both images share a center with no page overflow at 372 pixels. |
| Starting main CI | All nine jobs passed in [run 37500111063](https://github.com/cloudy-liu/ctty7/actions/runs/37500111063). This is the source baseline, not CI for the unpushed preparation. |
| Refreshed main CI | All nine jobs passed in [run 37644303956](https://github.com/cloudy-liu/ctty7/actions/runs/37644303956) on `8fcc511a`. Release PR CI is tracked separately. |

The Windows full suite needs Git for Windows' `usr/bin` utilities in its process
PATH. The first run exposed the CLI test's hardcoded `sh`; later core tests also
needed the existing `cat` and `sleep` utilities. The passing run used that PATH.

`cargo build --locked --workspace` compiled and linked the new binaries, but its
final copy to `target/debug/tty7-app.exe` was blocked by an existing development
daemon hosting two live panes. That daemon was left running. The newly linked
`target/debug/deps/tty7_app.exe` was copied into the isolated review runtime for
native checks. This is not a packaged release build.

## Publication after review

1. Recheck fork main, open PRs/issues, and the latest published custom tag. Review any newer changes before including them.
2. The user authorized publication on 2026-10-08. Commit and push to `fork`; the release preparation PR must target `cloudy-liu/ctty7`, base `main`.
3. Close #100 as completed after the accepted verification/docs are available on GitHub. Do not close unrelated issues or create an upstream PR.
4. Run CI on the approved release commit. Remove the README candidate notice when finalizing the publication and update the dated project status as appropriate.
5. Push matching tag `v0.2.0` to `fork` to build a draft release. Require the complete Windows, Linux, macOS, and remote-server asset set.
6. Download the draft's 12 program assets and `checksums.txt` into a fresh directory. Run `python .github/scripts/verify-release-assets.py <directory> 0.2.0 --checksums` and review the platform packaging checks.
7. Remove only resolved verification gaps from the notes. Keep real untested upgrades and platform checks under Known limitations. Use the English notes for the release body; any release commit body must also be English.
8. Publish explicitly and set Latest only after all assets pass. Verify the latest-release redirect and update package URLs. Never replace a published tag or binary.

The multi-platform packages, checksum set, and v0.2.0 in-place upgrade paths have
not been verified. Tag creation and publication follow the successful candidate
checks; authorization alone does not establish their verification.
