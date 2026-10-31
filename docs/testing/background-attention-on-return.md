# Background attention clears on application return

## Release provenance

The v0.9.0 tag does not include local commit `57a2e4a5` from
`fix/background-attention-on-return`. The
[v0.9.0 local-change audit](local-change-audit-v0.9.0.md) explicitly preserved
that branch outside the release, without an open PR. Consequently, the release
still computes application attention from waiting Agents and unread results.

This follow-up reapplies that change to fork main `1063bc9b`. A live completion
or input request while all ctty7 windows are inactive creates an application
reminder. Activating any workspace window acknowledges it immediately, even
when another tab is selected. The unread result and waiting status remain on
their original panes. Leaving again does not revive the acknowledged reminder;
a new background event can raise another one. Sound and toast preferences do
not control this application reminder.

## Regression

Run from the checkout with its normal Cargo target directory, or append
`--target-dir D:/gh-prj/ctty7/target` when sharing the existing local build cache:

```powershell
cargo test --offline --locked --bin tty7-app returning_to_another_tab_clears_the_app_badge_but_keeps_the_result_unread -- --nocapture
```

With only the regression added to main, the test failed in 0.19 seconds at:

```text
returning to the app must clear its badge without visiting the result
test result: FAILED. 0 passed; 1 failed
```

The baseline assertion reads production Agent rows and their actual attention
aggregation. The corrected test reads the production application snapshot.
Both send Agent reports through the real terminal stream and activate real
GPUI test windows; neither assigns the attention or unread state directly.

Coverage includes return to another tab, preserving its unread result, leaving
again, a later background input request, return through another workspace
window, foreground completion in another window, and another background turn.
An initial restored waiting report remains a silent baseline. Both sound and
toast channels are disabled in this regression.

## Verification on 2026-11-01

- The focused regression passed after the correction; the final complete GUI
  test run passed with **1,829 passed, 7 ignored, 0 failed** in 46.76 seconds.
- The first complete run had 1,828 passing tests and one failure in
  `editor_binary_notice_can_be_muted_without_hiding_open_errors`, at its
  simulated confirmation click. That test passed alone in 0.45 seconds, and
  the subsequent complete run passed. No editor code was changed.
- `cargo check --offline --locked --bin tty7-app --features markdown-visual-tests`
  passed, including the native fixture's switch to the production application
  attention snapshot.
- `cargo fmt --all -- --check`, `git diff --check`, and the Host boundary check
  passed. Existing compiler warnings remain.
- Independent Standards and Spec reviews against fork main `1063bc9b`
  reported zero actionable findings on both axes. Native visual acceptance
  remains unverified and requires a draft PR.
- The primary checkout and original `fix/background-attention-on-return`
  worktree were preserved. The follow-up lives in
  `fix/background-attention-return-v090` for independent review and delivery.

## Known limitations

Fresh native Before/After taskbar and tray captures are unverified. LCU reached
the baseline fixture and minimized it; the fixture observed a background Done
report with `attention: true`, `unread: true`, and both windows inactive.
Restoring that minimized fixture failed because its bound surface lacked an
activation operation. A subsequent attempt to select the background Chrome
window stopped Computer Use for the turn because the current browser URL could
not be determined with sufficient confidence to enforce policy. No further UI
input was attempted. Historical screenshots in
[Agent completion badge acceptance](agent-attention-badge.md) describe the old
unread-based behavior and are not visual evidence for this follow-up.

Any PR for this change must remain a draft until matched native screenshots
resolve visual acceptance or the limitation is explicitly accepted. This
follow-up does not replace the published v0.9.0 package or modify an installed
application.
