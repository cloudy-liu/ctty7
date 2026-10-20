# Agent completion badge acceptance

Agent results that have not been viewed leave a dark circular mark with a
white dot on the Windows taskbar and tray icons. There is no numeric count.
Viewing the result clears its unread state; another unread pane keeps the
application reminder visible. Existing waiting-for-input attention is retained.
Sounds and desktop notification preferences are independent and unchanged.

## Native Windows before and after

These are lossless crops of Windows system screenshots taken on 2026-10-20,
not HTML renders or repainted icons. The baseline is `f16436fa`, with only the
same opt-in screenshot fixture added. The changed build runs the same fixture,
window size, display scaling and Agent status sequence.

| Location | Before, completed result unread | After, completed result unread | After viewing result |
| --- | --- | --- | --- |
| Taskbar | ![No taskbar completion badge](images/agent-attention/before-taskbar.png) | ![Native dark taskbar badge with white dot](images/agent-attention/after-taskbar.png) | ![Native taskbar badge cleared](images/agent-attention/read-taskbar.png) |
| Tray | ![No tray completion badge](images/agent-attention/before-tray.png) | ![Native dark tray badge with white dot](images/agent-attention/after-tray.png) | Windows returned the inactive icon to its overflow area after the result was viewed. |

On the Windows 11 system used for these captures, `ITaskbarList3::SetOverlayIcon`
places the mark at the upper-right. Windows controls placement and scaling;
older taskbars can position overlays differently. The app supplies the same
circle-and-dot artwork used by the tray. No taskbar setting is changed.

## Reproduce

Build the opt-in native fixture:

```powershell
cargo build --locked --bin tty7-app --features markdown-visual-tests
```

Save a UTF-8 JSON file without a BOM, using absolute paths:

```json
{
  "sidebar": true,
  "attention_badge": true,
  "output": "C:/temp/ctty7-attention/after.png",
  "hold_seconds": 120
}
```

Launch with isolated configuration:

```powershell
target/debug/tty7-app.exe --config-dir C:/temp/ctty7-attention/config --ssh-tab-visual C:/temp/ctty7-attention/options.json
```

The fixture uses production terminal views, status messages, tray polling and
native taskbar delivery. It opens no real Agent or SSH connection. The first
pane reports Working then Done while another pane is selected. Wait for
`NATIVE_ATTENTION_BADGE_UNREAD`, then use Windows Screenshot Tool in Full screen
mode. Screenshot Tool's auto-saved PNG includes the actual taskbar and tray.

Create `after.read` next to the configured `after.png` path to focus the
completed pane through the normal application focus path. Wait for
`NATIVE_ATTENTION_BADGE_READ` and at least one tray polling interval before
capturing again. Remove the control file before another run. The fixture exits
after its hold period and does not write real workspace/session settings.

## Verification and limits

- The new lifecycle regression failed before the focus correction: a selected
  pane in an inactive window incorrectly reported its result as already read.
  It passes after checking both window activation and pane focus, including
  clearing on return to the selected pane.
- Foreground tests explicitly activate the window as well as focus its pane.
  GPUI test windows start inactive; pane focus alone does not establish the
  foreground case. Cross-platform tests cover both foreground and background
  completion, including their activation/focus preconditions.
- Notification/channel/focus integration tests and tray rendering/state tests
  pass, including the dark circle, white center and transparent outer corners.
- The Windows production and native-fixture paths compile. Taskbar updates run
  independently of the tray visibility preference and reapply during the
  existing one-second poll, including after Explorer recreates taskbar buttons.
- Unread state remains attached to live panes. Closing/reconstructing a
  workspace follows existing read-state semantics; this change adds no saved
  notification history.
- Native visual acceptance covers this Windows 11 environment. Other taskbar
  versions, Explorer restart recovery, and macOS/Linux visuals were not
  exercised interactively. macOS has no new Dock badge implementation.
