# Open issue audit on 2026-10-23

Checked the three open issues against fork main `b0b57316`, their merged PRs,
the implementation, and the native artifacts from
[main CI](https://github.com/cloudy-liu/ctty7/actions/runs/39921343329).

## Disposition

| Issue | Implementation already on main | Remaining work |
| --- | --- | --- |
| [#182](https://github.com/cloudy-liu/ctty7/issues/182), multi-file document tabs | [#186](https://github.com/cloudy-liu/ctty7/pull/186) closes child issues #183, #184 and #185, but only references the parent spec. [#191](https://github.com/cloudy-liu/ctty7/pull/191) adds HTML reading-position preservation. | This follow-up fixes the same-document fragment case found during the audit and links the parent for closure when the PR merges. |
| [#175](https://github.com/cloudy-liu/ctty7/issues/175), native HTML preview | [#178](https://github.com/cloudy-liu/ctty7/pull/178) implements WKWebView and shared document behavior. Windows preview already exists; Linux integration boundaries are documented. The PR uses `Refs`, so merging it did not close the spec. | Keep open for the outstanding device, resource/performance and packaged-runtime acceptance described below. |
| [#188](https://github.com/cloudy-liu/ctty7/issues/188), upper-right taskbar attention | [#189](https://github.com/cloudy-liu/ctty7/pull/189) composes attention into both window icon slots. It does not contain a closing reference. | Keep open for actual Explorer acceptance in horizontal/vertical layouts, orientation changes, pinned/grouped buttons and taskbar recreation. |

The open issue count therefore does not describe three missing implementations.
The document spec has an uncovered HTML edge case. The other two issues retain
acceptance work that merged code and unit tests cannot establish.

## HTML fragment regression

Following a page link changes `file:///.../report.HTML` to
`file:///.../report.HTML#reading-position`. Scroll capture previously compared
the entire browser URL with the saved file URL, so it discarded a valid
600-CSS-pixel position. Returning from another HTML tab restored the top instead.
The restoration guard also compared the entire URL, affecting refresh after
fragment navigation.

Ignore the fragment in both browser-owned URL checks, while retaining the
file path and query. Encoded `#` characters in filenames remain `%23` and are
part of the file identity. The existing native fixture follows an actual
same-document link, switches files, refreshes the fragment-bearing page and
checks the observable viewport and continued preview availability. Wait for
refresh restoration before starting the next interaction so the test does not
race an earlier restore.

Reproduction uses the production document opening and preview actions:

```powershell
cargo build --locked --features markdown-visual-tests --bin tty7-app
python scripts/html-native-preview.py target/debug/tty7-app.exe OUTPUT_DIRECTORY
```

The native regression was observed red in two independent stages:

- Original URL checks: `HTML reading position was lost across file tabs`.
  The browser returned 600 CSS pixels, but the fragment-bearing URL caused
  the document panel to discard the snapshot.
- Capture normalized, restoration unchanged: the file switch restored 600,
  but refreshing the fragment-bearing page failed with
  `HTML reading position 0 was not restored` and fell back to source.

With both guards normalized, the complete Windows acceptance passes and
reports `fragment_scroll_and_refresh: true`. It also passes existing resource,
session ownership, browser recreation, rapid-switch, close/reopen and failure
recovery checks. The input filename is now `report #1.HTML`, verifying `%23`
in the file path alongside Unicode, spaces and an uppercase extension. This
run used Windows 11 build 22000.2538, WebView2 155 and DPR 1.75. These checks
establish behavior, not a performance improvement.

Final local checks pass: `cargo test --workspace --locked` reports 3,157 passed
and 12 ignored across its standard test summaries; the native visual-feature
build, `cargo fmt --all -- --check`, `git diff --check` and the Host boundary
check also pass. Temporary diagnostic logging was removed before the passing
native run.

## Existing macOS evidence inspected

Downloaded and inspected both `native-html-aarch64-apple-darwin` and
`native-html-x86_64-apple-darwin` artifacts from the main CI run linked above.
Both report successful relative CSS/JavaScript/SVG loading, script interaction,
refresh, singleton reuse, per-file/session scrolling, browser recreation,
rapid selection, close/reopen and deleted-file source recovery. The native
desktop images visibly show the HTML child view, loaded resources, file tabs,
breadcrumbs and Preview/HTML controls on both architectures.

These are observations of the existing main commit, not evidence for the new
fragment regression. DOM script clicks do not establish native mouse/keyboard
focus behavior. The single polling-based timing samples do not establish
first visible paint, release-profile performance or comparative resource use.
Issue #175 still needs its native input/layout/occlusion/DPI matrix, comparable
process-group resource measurements and actual packaged-runtime checks.

## Taskbar evidence

The existing ten tray tests pass on this Windows machine, including native
window icon installation/restoration and upper-right artwork at multiple
sizes. Their results establish handle/pixel behavior, not Explorer rendering.
The taskbar document already marks its older screenshots as historical.
No fresh vertical-taskbar screenshot or orientation-change result was obtained
during this audit. Issue #188 therefore remains open for acceptance rather than
receiving another copy of the already-merged icon implementation.
